use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream, StreamConfig};
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::{Split, Producer, Consumer}};
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use tauri::AppHandle;
use tauri::Emitter;

/// 音频控制命令枚举，用于音频线程之间的消息传递
enum AudioCmd {
    Play(String),
    PlayLocal(String),
    Pause,
    Resume,
    Stop,
    Seek(f64),
    SetVolume(f32),
    SetDevice(Option<String>),
}

/// 音频控制器，通过通道向音频线程发送控制命令
pub struct AudioController {
    tx: Sender<AudioCmd>,
    current_url: Arc<Mutex<Option<String>>>,
    position: Arc<Mutex<f64>>,
    is_playing: Arc<AtomicBool>,
}

impl AudioController {
    /// 创建新的音频控制器，并启动后台音频线程
    pub fn new(app_handle: AppHandle) -> Self {
        let (tx, rx) = channel();
        let current_url = Arc::new(Mutex::new(None));
        let position = Arc::new(Mutex::new(0.0));
        let is_playing = Arc::new(AtomicBool::new(false));
        let url_clone = current_url.clone();
        let pos_clone = position.clone();
        let playing_clone = is_playing.clone();
        let ah_clone = app_handle.clone();
        thread::spawn(move || audio_thread(rx, url_clone, pos_clone, playing_clone, ah_clone));
        AudioController { tx, current_url, position, is_playing }
    }

    /// 播放指定URL的网络音频
    pub fn play_url(&self, url: &str) {
        *self.current_url.lock().unwrap() = Some(url.to_string());
        let _ = self.tx.send(AudioCmd::Play(url.to_string()));
    }
    /// 播放指定路径的本地音频文件
    pub fn play_local(&self, path: &str) {
        *self.current_url.lock().unwrap() = Some(path.to_string());
        let _ = self.tx.send(AudioCmd::PlayLocal(path.to_string()));
    }
    /// 暂停当前播放
    pub fn pause(&self) { let _ = self.tx.send(AudioCmd::Pause); }
    /// 恢复播放
    pub fn resume(&self) { let _ = self.tx.send(AudioCmd::Resume); }
    /// 停止当前播放
    pub fn stop(&self) { let _ = self.tx.send(AudioCmd::Stop); }
    /// 设置音频输出设备，传入 None 则使用系统默认设备
    pub fn set_device(&self, device: Option<String>) {
        let _ = self.tx.send(AudioCmd::SetDevice(device));
    }
    /// 跳转到指定时间位置（秒）
    pub fn seek(&self, time: f64) { let _ = self.tx.send(AudioCmd::Seek(time)); }
    /// 设置播放音量，范围 0.0 ~ 1.0
    pub fn set_volume(&self, vol: f32) { let _ = self.tx.send(AudioCmd::SetVolume(vol)); }
    /// 获取当前播放位置（秒）
    pub fn get_position(&self) -> f64 {
        *self.position.lock().unwrap()
    }
    pub fn get_is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }
}

/// 缓冲区内部状态，存储已下载的字节数据及完成/取消标志
struct BufferState {
    bytes: Vec<u8>,
    done: bool,
    cancelled: bool,
}

/// 线程安全的共享缓冲区，支持生产者写入和消费者读取的同步等待
struct SharedBuffer {
    state: Mutex<BufferState>,
    available: Condvar,
}

impl SharedBuffer {
    /// 创建新的空共享缓冲区
    fn new() -> Self {
        SharedBuffer {
            state: Mutex::new(BufferState {
                bytes: Vec::new(),
                done: false,
                cancelled: false,
            }),
            available: Condvar::new(),
        }
    }

    /// 向缓冲区追加写入一块数据，并通知等待的读取者
    fn write_chunk(&self, chunk: &[u8]) {
        let mut state = self.state.lock().unwrap();
        state.bytes.extend_from_slice(chunk);
        self.available.notify_all();
    }

    /// 标记缓冲区写入已完成，通知读取者不再有新数据
    fn mark_done(&self) {
        let mut state = self.state.lock().unwrap();
        state.done = true;
        self.available.notify_all();
    }

    /// 取消缓冲区，中断正在进行的读写操作
    fn cancel(&self) {
        let mut state = self.state.lock().unwrap();
        state.cancelled = true;
        self.available.notify_all();
    }

    /// 返回已缓冲的数据字节数
    fn len(&self) -> usize {
        self.state.lock().unwrap().bytes.len()
    }

    /// 检查缓冲区是否已标记为写入完成
    fn is_done(&self) -> bool {
        self.state.lock().unwrap().done
    }

    /// 检查缓冲区是否已被取消
    fn is_cancelled(&self) -> bool {
        self.state.lock().unwrap().cancelled
    }
}

/// 流式读取器，从共享缓冲区中按需读取数据，实现 `Read` 和 `Seek` trait
struct StreamingReader {
    buffer: Arc<SharedBuffer>,
    pos: usize,
}

impl StreamingReader {
    /// 创建新的流式读取器，绑定到指定的共享缓冲区
    fn new(buffer: Arc<SharedBuffer>) -> Self {
        StreamingReader { buffer, pos: 0 }
    }
}

impl Read for StreamingReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut state = self.buffer.state.lock().unwrap();
        loop {
            let available = state.bytes.len().saturating_sub(self.pos);
            if available > 0 {
                let to_read = std::cmp::min(buf.len(), available);
                buf[..to_read].copy_from_slice(&state.bytes[self.pos..self.pos + to_read]);
                self.pos += to_read;
                return Ok(to_read);
            }
            if state.done {
                return Ok(0);
            }
            if state.cancelled {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "cancelled"));
            }
            let result = self
                .buffer
                .available
                .wait_timeout(state, Duration::from_millis(500))
                .unwrap();
            state = result.0;
        }
    }
}

impl std::io::Seek for StreamingReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::Current(offset) => self.pos as i64 + offset,
            std::io::SeekFrom::End(offset) => {
                let mut state = self.buffer.state.lock().unwrap();
                loop {
                    if state.done {
                        break state.bytes.len() as i64 + offset;
                    }
                    if state.cancelled {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Interrupted,
                            "cancelled",
                        ));
                    }
                    let result = self
                        .buffer
                        .available
                        .wait_timeout(state, Duration::from_millis(500))
                        .unwrap();
                    state = result.0;
                }
            }
        };
        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        let mut state = self.buffer.state.lock().unwrap();
        loop {
            if new_pos as usize <= state.bytes.len() {
                self.pos = new_pos as usize;
                return Ok(self.pos as u64);
            }
            if state.done {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "seek past end",
                ));
            }
            if state.cancelled {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "cancelled",
                ));
            }
            let result = self
                .buffer
                .available
                .wait_timeout(state, Duration::from_millis(500))
                .unwrap();
            state = result.0;
        }
    }
}

impl MediaSource for StreamingReader {
    fn is_seekable(&self) -> bool { true }
    fn byte_len(&self) -> Option<u64> { None }
}

/// 流式下载音频数据到共享缓冲区，支持下载进度事件通知
fn download_audio_streaming(
    url: &str,
    buffer: &SharedBuffer,
    app_handle: &AppHandle,
) -> Result<(), String> {
    let resp = reqwest::blocking::get(url).map_err(|e| format!("下载失败: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP 错误: {}", resp.status()));
    }
    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut reader = resp;
    loop {
        if buffer.is_cancelled() {
            return Err("下载已取消".to_string());
        }
        let mut chunk = [0u8; 8192];
        let read_size = reader
            .read(&mut chunk)
            .map_err(|e| format!("读取失败: {}", e))?;
        if read_size == 0 {
            break;
        }
        buffer.write_chunk(&chunk[..read_size]);
        downloaded += read_size as u64;
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        let _ = app_handle.emit("cache-progress", progress);
    }
    Ok(())
}

/// 初始缓冲区大小，达到此字节数后才开始播放
const INITIAL_BUFFER_SIZE: usize = 65536;
/// 环形缓冲区容量（采样数），约 4 秒的 48kHz 立体声数据
const RING_BUFFER_SAMPLES: usize = 48000 * 4;

/// 播放状态，记录当前播放的运行时信息
struct PlaybackState {
    playing: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    decode_done: Arc<AtomicBool>,
    buffer_exhausted: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    sample_rate: u32,
    channels: u16,
    samples_played: Arc<AtomicU64>,
    start_time: f64,
}

impl PlaybackState {
    /// 根据已播放采样数计算当前播放位置（秒）
    fn position(&self) -> f64 {
        let samples = self.samples_played.load(Ordering::Relaxed) as f64;
        self.start_time + samples / (self.sample_rate as f64 * self.channels as f64)
    }
}

/// 输出上下文，持有音频输出流和解码线程的句柄
struct OutputContext {
    _stream: Stream,
    _decode_thread: thread::JoinHandle<()>,
    playback: PlaybackState,
}

/// 将 Symphonia 解码后的音频缓冲区转换为交错排列的 f32 采样数据
fn convert_to_interleaved_f32(decoded: &AudioBufferRef) -> Vec<f32> {
    let channels = decoded.spec().channels.count();
    let frames = decoded.frames();
    let mut out = Vec::with_capacity(frames * channels);

    match decoded {
        AudioBufferRef::U8(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / u8::MAX as f32 * 2.0 - 1.0);
                }
            }
        }
        AudioBufferRef::U16(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / u16::MAX as f32 * 2.0 - 1.0);
                }
            }
        }
        AudioBufferRef::U24(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame].0 as f32 / 8388607.0);
                }
            }
        }
        AudioBufferRef::U32(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / u32::MAX as f32 * 2.0 - 1.0);
                }
            }
        }
        AudioBufferRef::S8(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / i8::MAX as f32);
                }
            }
        }
        AudioBufferRef::S16(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / i16::MAX as f32);
                }
            }
        }
        AudioBufferRef::S24(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame].0 as f32 / 8388607.0);
                }
            }
        }
        AudioBufferRef::S32(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / i32::MAX as f32);
                }
            }
        }
        AudioBufferRef::F32(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame]);
                }
            }
        }
        AudioBufferRef::F64(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32);
                }
            }
        }
    }

    out
}

/// 重混声道数，将交错采样数据从源声道数转换为目标声道数
fn remix_channels(
    interleaved: &[f32],
    src_channels: u16,
    target_channels: u16,
    src_frames: usize,
) -> Vec<f32> {
    if src_channels == target_channels {
        return interleaved.to_vec();
    }

    let src_ch = src_channels as usize;
    let tgt_ch = target_channels as usize;
    let mut out = Vec::with_capacity(src_frames * tgt_ch);

    if src_ch == 1 && tgt_ch == 2 {
        for &s in interleaved {
            out.push(s);
            out.push(s);
        }
    } else if src_ch == 2 && tgt_ch == 1 {
        for i in 0..src_frames {
            let l = interleaved[i * 2];
            let r = interleaved[i * 2 + 1];
            out.push((l + r) * 0.5);
        }
    } else {
        for i in 0..src_frames {
            for ch in 0..tgt_ch {
                let src_ch_idx = ch.min(src_ch.saturating_sub(1));
                out.push(interleaved[i * src_ch + src_ch_idx]);
            }
        }
    }

    out
}

/// 对解码音频进行重采样和声道重混，输出目标采样率和声道数的交错 f32 数据
fn resample_and_remix(
    decoded: &AudioBufferRef,
    target_sample_rate: u32,
    target_channels: u16,
    src_rate: f64,
    src_channels: u16,
) -> Vec<f32> {
    let interleaved = convert_to_interleaved_f32(decoded);
    let src_frames = if src_channels > 0 {
        interleaved.len() / src_channels as usize
    } else {
        0
    };

    if src_frames == 0 {
        return Vec::new();
    }

    let remixed = remix_channels(&interleaved, src_channels, target_channels, src_frames);
    let remixed_ch = target_channels as usize;

    let ratio = target_sample_rate as f64 / src_rate;
    let need_resample = (ratio - 1.0).abs() > 0.001;

    if !need_resample {
        return remixed;
    }

    let target_frames = (src_frames as f64 * ratio).round() as usize;
    if target_frames == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(target_frames * remixed_ch);
    for i in 0..target_frames {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;
        let next_idx = (src_idx + 1).min(src_frames - 1);

        for ch in 0..remixed_ch {
            let s0 = remixed[src_idx * remixed_ch + ch];
            let s1 = remixed[next_idx * remixed_ch + ch];
            out.push(s0 + (s1 - s0) * frac as f32);
        }
    }

    out
}

/// 将音频数据解码并写入环形缓冲区，供播放回调消费
fn decode_to_ring(
    mss: MediaSourceStream,
    mut producer: HeapProd<f32>,
    playing: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    decode_done: Arc<AtomicBool>,
    seek_time: Option<f64>,
    target_sample_rate: u32,
    target_channels: u16,
) {
    let hint = Hint::new();
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[audio] 探测格式失败: {}", e);
            decode_done.store(true, Ordering::Relaxed);
            return;
        }
    };

    let mut format_reader = probed.format;
    let track = match format_reader
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    {
        Some(t) => t,
        None => {
            eprintln!("[audio] 未找到有效音频轨道");
            decode_done.store(true, Ordering::Relaxed);
            return;
        }
    };

    let track_id = track.id;
    let codec_params = &track.codec_params;
    let src_rate = codec_params.sample_rate.unwrap_or(44100) as f64;
    let src_channels = codec_params.channels.unwrap_or_else(|| {
        symphonia::core::audio::Channels::FRONT_LEFT | symphonia::core::audio::Channels::FRONT_RIGHT
    }).count() as u16;

    let mut decoder = match symphonia::default::get_codecs().make(codec_params, &decoder_opts) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[audio] 创建解码器失败: {}", e);
            decode_done.store(true, Ordering::Relaxed);
            return;
        }
    };

    if let Some(time) = seek_time {
        let seek_to = SeekTo::Time {
            time: Time::from(time),
            track_id: Some(track_id),
        };
        let _ = format_reader.seek(SeekMode::Accurate, seek_to);
    }

    let ratio = target_sample_rate as f64 / src_rate;
    let need_resample = (ratio - 1.0).abs() > 0.001;
    let need_remix = src_channels != target_channels;

    while !cancelled.load(Ordering::Relaxed) {
        let packet = match format_reader.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => continue,
            Err(e) => {
                eprintln!("[audio] 读取包失败: {}", e);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[audio] 解码失败: {}", e);
                continue;
            }
        };

        let samples = if need_resample || need_remix {
            resample_and_remix(&decoded, target_sample_rate, target_channels, src_rate, src_channels)
        } else {
            convert_to_interleaved_f32(&decoded)
        };

        let mut write_pos = 0;
        while write_pos < samples.len() && !cancelled.load(Ordering::Relaxed) {
            let remaining = &samples[write_pos..];
            let n = producer.push_slice(remaining);
            if n == 0 {
                if !playing.load(Ordering::Relaxed) {
                    while !playing.load(Ordering::Relaxed) && !cancelled.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            write_pos += n;
        }
    }

    decode_done.store(true, Ordering::Relaxed);
}

/// 启动音频播放，创建解码线程和 cpal 输出流
fn start_playback(
    mss: MediaSourceStream,
    device: &cpal::Device,
    current_volume: f32,
    seek_time: Option<f64>,
) -> Result<OutputContext, String> {
    let default_config = device
        .default_output_config()
        .map_err(|e| format!("获取设备配置失败: {}", e))?;

    let sr = default_config.sample_rate().0;
    let ch = default_config.channels();
    let sample_format = default_config.sample_format();

    let rb = HeapRb::<f32>::new(RING_BUFFER_SAMPLES);
    let (producer, consumer) = rb.split();

    let playing = Arc::new(AtomicBool::new(true));
    let cancelled = Arc::new(AtomicBool::new(false));
    let decode_done = Arc::new(AtomicBool::new(false));
    let buffer_exhausted = Arc::new(AtomicBool::new(false));
    let volume = Arc::new(Mutex::new(current_volume));
    let samples_played = Arc::new(AtomicU64::new(0));
    let start_time = seek_time.unwrap_or(0.0);

    let playing_clone = playing.clone();
    let cancelled_clone = cancelled.clone();
    let decode_done_clone = decode_done.clone();
    let decode_handle = thread::spawn(move || {
        decode_to_ring(
            mss,
            producer,
            playing_clone,
            cancelled_clone,
            decode_done_clone,
            seek_time,
            sr,
            ch,
        );
    });

    let stream = build_cpal_stream(device, sr, ch, sample_format, consumer, volume.clone(), playing.clone(), samples_played.clone(), decode_done.clone(), buffer_exhausted.clone())?;
    stream.play().map_err(|e| format!("播放流失败: {}", e))?;

    Ok(OutputContext {
        _stream: stream,
        _decode_thread: decode_handle,
        playback: PlaybackState {
            playing,
            cancelled,
            decode_done,
            buffer_exhausted,
            volume,
            sample_rate: sr,
            channels: ch,
            samples_played,
            start_time,
        },
    })
}

/// 构建 cpal 音频输出流，支持 f32、i16、u16 三种采样格式
fn build_cpal_stream(
    device: &cpal::Device,
    sample_rate: u32,
    channels: u16,
    sample_format: cpal::SampleFormat,
    mut consumer: HeapCons<f32>,
    volume: Arc<Mutex<f32>>,
    playing: Arc<AtomicBool>,
    samples_played: Arc<AtomicU64>,
    decode_done: Arc<AtomicBool>,
    buffer_exhausted: Arc<AtomicBool>,
) -> Result<Stream, String> {
    let config = StreamConfig {
        channels,
        sample_rate: SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let err_fn = |err: cpal::StreamError| eprintln!("[audio] 输出错误: {}", err);

    match sample_format {
        cpal::SampleFormat::F32 => {
            let sp = samples_played;
            let dd = decode_done;
            let be = buffer_exhausted;
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        if !playing.load(Ordering::Relaxed) {
                            data.fill(0.0);
                            return;
                        }
                        let vol = *volume.lock().unwrap();
                        let read = consumer.pop_slice(data);
                        for (i, s) in data.iter_mut().enumerate() {
                            if i < read {
                                *s *= vol;
                            } else {
                                *s = 0.0;
                            }
                        }
                        sp.fetch_add(read as u64, Ordering::Relaxed);
                        if read == 0 && dd.load(Ordering::Relaxed) {
                            be.store(true, Ordering::Relaxed);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("创建输出流失败: {}", e))
        }

        cpal::SampleFormat::I16 => {
            let mut f32_buf: Vec<f32> = Vec::new();
            let sp = samples_played;
            let dd = decode_done;
            let be = buffer_exhausted;
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        if !playing.load(Ordering::Relaxed) {
                            data.fill(0);
                            return;
                        }
                        let vol = *volume.lock().unwrap();
                        if f32_buf.len() != data.len() {
                            f32_buf.resize(data.len(), 0.0);
                        }
                        let read = consumer.pop_slice(&mut f32_buf);
                        for (i, s) in data.iter_mut().enumerate() {
                            if i < read {
                                *s = (f32_buf[i] * vol * 32767.0)
                                    .clamp(-32768.0, 32767.0) as i16;
                            } else {
                                *s = 0;
                            }
                        }
                        sp.fetch_add(read as u64, Ordering::Relaxed);
                        if read == 0 && dd.load(Ordering::Relaxed) {
                            be.store(true, Ordering::Relaxed);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("创建输出流失败: {}", e))
        }

        cpal::SampleFormat::U16 => {
            let mut f32_buf: Vec<f32> = Vec::new();
            let sp = samples_played;
            let dd = decode_done;
            let be = buffer_exhausted;
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        if !playing.load(Ordering::Relaxed) {
                            data.fill(32768);
                            return;
                        }
                        let vol = *volume.lock().unwrap();
                        if f32_buf.len() != data.len() {
                            f32_buf.resize(data.len(), 0.0);
                        }
                        let read = consumer.pop_slice(&mut f32_buf);
                        for (i, s) in data.iter_mut().enumerate() {
                            if i < read {
                                *s = ((f32_buf[i] * vol + 1.0) * 32767.5)
                                    .clamp(0.0, 65535.0) as u16;
                            } else {
                                *s = 32768;
                            }
                        }
                        sp.fetch_add(read as u64, Ordering::Relaxed);
                        if read == 0 && dd.load(Ordering::Relaxed) {
                            be.store(true, Ordering::Relaxed);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("创建输出流失败: {}", e))
        }

        _ => Err(format!("不支持的采样格式: {:?}", sample_format)),
    }
}

/// 获取系统默认输出设备的名称
fn get_system_default_device_name() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|d| d.name().ok())
}

/// 列出系统中所有可用的音频输出设备名称（去重排序后）
pub fn list_output_devices() -> Vec<String> {
    let host = cpal::default_host();
    if let Ok(devices) = host.output_devices() {
        let mut names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();
        names.sort();
        names.dedup();
        names
    } else {
        vec![]
    }
}

/// 按名称查找音频输出设备，未找到则返回 None
fn find_device_by_name(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    if let Ok(devices) = host.output_devices() {
        for d in devices {
            if let Ok(n) = d.name() {
                if n == name {
                    return Some(d);
                }
            }
        }
    }
    None
}

/// 获取音频输出设备，优先使用指定名称的设备，否则回退到系统默认设备
fn get_output_device(selected: &Option<String>) -> cpal::Device {
    match selected {
        Some(name) => find_device_by_name(name).unwrap_or_else(|| {
            eprintln!("[audio] 未找到设备 `{}`，回退默认", name);
            cpal::default_host()
                .default_output_device()
                .expect("无可用音频设备")
        }),
        None => cpal::default_host()
            .default_output_device()
            .expect("无可用音频设备"),
    }
}

/// 停止播放，取消解码并重置共享播放位置
fn stop_playback(output_ctx: &mut Option<OutputContext>, shared_position: &Arc<Mutex<f64>>) {
    if let Some(ref mut ctx) = output_ctx {
        ctx.playback.cancelled.store(true, Ordering::Relaxed);
        ctx.playback.playing.store(false, Ordering::Relaxed);
    }
    *output_ctx = None;
    *shared_position.lock().unwrap() = 0.0;
}

fn rebuild_mss(
    local_path: &Option<String>,
    audio_buffer: &Option<Arc<SharedBuffer>>,
) -> Option<MediaSourceStream> {
    if let Some(ref path) = local_path {
        let file = std::fs::File::open(path).ok()?;
        Some(MediaSourceStream::new(Box::new(file), Default::default()))
    } else if let Some(ref buffer) = audio_buffer {
        let reader = StreamingReader::new(buffer.clone());
        Some(MediaSourceStream::new(Box::new(reader), Default::default()))
    } else {
        None
    }
}

fn restart_playback_on_device_change(
    output_ctx: &mut Option<OutputContext>,
    shared_position: &Arc<Mutex<f64>>,
    local_path: &Option<String>,
    audio_buffer: &Option<Arc<SharedBuffer>>,
    selected_device: &Option<String>,
    current_volume: f32,
    audio_paused: bool,
) -> Result<OutputContext, String> {
    let current_pos = output_ctx
        .as_ref()
        .map(|ctx| ctx.playback.position())
        .unwrap_or(0.0);
    let was_paused = audio_paused;
    stop_playback(output_ctx, shared_position);

    let mss = rebuild_mss(local_path, audio_buffer)
        .ok_or_else(|| "无法重建音频源".to_string())?;
    let device = get_output_device(selected_device);

    let ctx = start_playback(mss, &device, current_volume, Some(current_pos))?;
    if was_paused {
        ctx.playback.playing.store(false, Ordering::Relaxed);
    }
    Ok(ctx)
}

/// 音频线程主循环，接收命令并管理播放生命周期，包括设备热切换和播放结束检测
fn audio_thread(rx: Receiver<AudioCmd>, _current_url: Arc<Mutex<Option<String>>>, shared_position: Arc<Mutex<f64>>, is_playing: Arc<AtomicBool>, app_handle: AppHandle) {
    let mut selected_device: Option<String> = None;
    let mut current_volume: f32 = 1.0;
    let mut output_ctx: Option<OutputContext> = None;
    let mut current_audio_buffer: Option<Arc<SharedBuffer>> = None;
    let mut current_local_path: Option<String> = None;
    let mut audio_active = false;
    let mut audio_paused = false;
    let mut manual_stop = false;
    let mut last_default_name = get_system_default_device_name();

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(cmd) => match cmd {
                AudioCmd::Play(url) => {
                    audio_active = false;
                    audio_paused = false;
                    is_playing.store(false, Ordering::Relaxed);
                    manual_stop = false;
                    current_local_path = None;

                    stop_playback(&mut output_ctx, &shared_position);
                    if let Some(ref buf) = current_audio_buffer {
                        buf.cancel();
                    }

                    let buffer = Arc::new(SharedBuffer::new());
                    current_audio_buffer = Some(buffer.clone());

                    let buffer_clone = buffer.clone();
                    let ah_clone = app_handle.clone();
                    let url_clone = url.clone();
                    thread::spawn(move || {
                        if let Err(e) = download_audio_streaming(&url_clone, &buffer_clone, &ah_clone) {
                            if !buffer_clone.is_cancelled() {
                                eprintln!("[audio] 流式下载失败: {}", e);
                            }
                        }
                        buffer_clone.mark_done();
                    });

                    loop {
                        let len = buffer.len();
                        if len >= INITIAL_BUFFER_SIZE || buffer.is_done() || buffer.is_cancelled() {
                            break;
                        }
                        thread::sleep(Duration::from_millis(50));
                    }

                    if buffer.is_cancelled() || buffer.len() == 0 {
                        current_audio_buffer = None;
                        continue;
                    }

                    let mss = MediaSourceStream::new(
                        Box::new(StreamingReader::new(buffer.clone())),
                        Default::default(),
                    );

                    let device = get_output_device(&selected_device);
                    match start_playback(mss, &device, current_volume, None) {
                        Ok(ctx) => {
                            output_ctx = Some(ctx);
                            audio_active = true;
                            is_playing.store(true, Ordering::Relaxed);
                            let _ = app_handle.emit("audio-started", ());
                        }
                        Err(e) => {
                            eprintln!("[audio] 播放启动失败: {}", e);
                        }
                    }
                }

                AudioCmd::PlayLocal(path) => {
                    audio_active = false;
                    audio_paused = false;
                    is_playing.store(false, Ordering::Relaxed);
                    manual_stop = false;
                    current_local_path = Some(path.clone());

                    stop_playback(&mut output_ctx, &shared_position);
                    if let Some(ref buf) = current_audio_buffer {
                        buf.cancel();
                    }

                    let file = match std::fs::File::open(&path) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("[audio] 打开本地文件失败: {}", e);
                            continue;
                        }
                    };

                    let buffer = Arc::new(SharedBuffer::new());
                    current_audio_buffer = Some(buffer.clone());

                    let mss = MediaSourceStream::new(Box::new(file), Default::default());

                    let device = get_output_device(&selected_device);
                    match start_playback(mss, &device, current_volume, None) {
                        Ok(ctx) => {
                            output_ctx = Some(ctx);
                            audio_active = true;
                            is_playing.store(true, Ordering::Relaxed);
                            let _ = app_handle.emit("audio-started", ());
                        }
                        Err(e) => {
                            eprintln!("[audio] 本地播放失败: {}", e);
                        }
                    }
                }

                AudioCmd::Pause => {
                    audio_paused = true;
                    is_playing.store(false, Ordering::Relaxed);
                    if let Some(ref ctx) = output_ctx {
                        ctx.playback.playing.store(false, Ordering::Relaxed);
                    }
                }

                AudioCmd::Resume => {
                    audio_paused = false;
                    if audio_active {
                        is_playing.store(true, Ordering::Relaxed);
                    }
                    if let Some(ref ctx) = output_ctx {
                        ctx.playback.playing.store(true, Ordering::Relaxed);
                    }
                }

                AudioCmd::Stop => {
                    audio_active = false;
                    audio_paused = false;
                    is_playing.store(false, Ordering::Relaxed);
                    manual_stop = true;
                    stop_playback(&mut output_ctx, &shared_position);
                    if let Some(ref buf) = current_audio_buffer {
                        buf.cancel();
                    }
                }

                AudioCmd::Seek(time) => {
                    stop_playback(&mut output_ctx, &shared_position);

                    let mss = match rebuild_mss(&current_local_path, &current_audio_buffer) {
                        Some(mss) => mss,
                        None => continue,
                    };

                    let device = get_output_device(&selected_device);
                    match start_playback(mss, &device, current_volume, Some(time)) {
                        Ok(ctx) => {
                            if audio_paused {
                                is_playing.store(false, Ordering::Relaxed);
                                ctx.playback.playing.store(false, Ordering::Relaxed);
                            } else {
                                is_playing.store(true, Ordering::Relaxed);
                            }
                            output_ctx = Some(ctx);
                            audio_active = true;
                        }
                        Err(e) => {
                            eprintln!("[audio] seek 播放失败: {}", e);
                        }
                    }
                }

                AudioCmd::SetVolume(vol) => {
                    current_volume = vol;
                    if let Some(ref ctx) = output_ctx {
                        *ctx.playback.volume.lock().unwrap() = vol;
                    }
                }

                AudioCmd::SetDevice(dev) => {
                    selected_device = dev;
                    if audio_active {
                        match restart_playback_on_device_change(
                            &mut output_ctx,
                            &shared_position,
                            &current_local_path,
                            &current_audio_buffer,
                            &selected_device,
                            current_volume,
                            audio_paused,
                        ) {
                            Ok(ctx) => { output_ctx = Some(ctx); }
                            Err(e) => { eprintln!("[audio] 设备切换失败: {}", e); }
                        }
                    }
                    if selected_device.is_none() {
                        last_default_name = get_system_default_device_name();
                    }
                }
            },

            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if audio_active {
                    if let Some(ref ctx) = output_ctx {
                        if ctx.playback.decode_done.load(Ordering::Relaxed)
                            && ctx.playback.buffer_exhausted.load(Ordering::Relaxed)
                            && !manual_stop && !audio_paused {
                            audio_active = false;
                            is_playing.store(false, Ordering::Relaxed);
                            let _ = app_handle.emit("audio-ended", ());
                        }
                        let pos = ctx.playback.position();
                        *shared_position.lock().unwrap() = pos;
                    }
                }

                if selected_device.is_none() {
                    let current_default = get_system_default_device_name();
                    if current_default != last_default_name {
                        println!(
                            "[audio] 系统默认设备变化: {:?} -> {:?}",
                            last_default_name, current_default
                        );
                        last_default_name = current_default;

                        if audio_active {
                            if let Ok(ctx) = restart_playback_on_device_change(
                                &mut output_ctx,
                                &shared_position,
                                &current_local_path,
                                &current_audio_buffer,
                                &selected_device,
                                current_volume,
                                audio_paused,
                            ) {
                                output_ctx = Some(ctx);
                            }
                        }
                    }
                }
            }

            Err(_) => break,
        }
    }
}

use tauri::State;
use std::sync::Mutex as StdMutex;

/// Tauri 管理的音频状态，内部包装 `AudioController` 的互斥锁
pub struct AppAudio(pub StdMutex<AudioController>);

/// Tauri 命令：播放网络音频
#[tauri::command]
pub fn play_audio(state: State<'_, AppAudio>, url: String) -> Result<(), String> {
    let ctrl = state.0.lock().map_err(|e| e.to_string())?;
    ctrl.play_url(&url);
    Ok(())
}

/// Tauri 命令：播放本地音频文件
#[tauri::command]
pub fn play_local_audio(state: State<'_, AppAudio>, path: String) -> Result<(), String> {
    let ctrl = state.0.lock().map_err(|e| e.to_string())?;
    ctrl.play_local(&path);
    Ok(())
}

/// Tauri 命令：暂停当前播放
#[tauri::command]
pub fn pause_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.pause(); }
}

/// Tauri 命令：恢复播放
#[tauri::command]
pub fn resume_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.resume(); }
}

/// Tauri 命令：停止当前播放
#[tauri::command]
pub fn stop_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.stop(); }
}

/// Tauri 命令：获取所有可用的音频输出设备列表
#[tauri::command]
pub fn get_output_devices() -> Vec<String> {
    list_output_devices()
}

/// Tauri 命令：设置音频输出设备，传入 None 使用系统默认设备
#[tauri::command]
pub fn set_output_device(state: State<'_, AppAudio>, device: Option<String>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.set_device(device); }
}

/// Tauri 命令：跳转到指定播放位置（秒）
#[tauri::command]
pub fn seek_audio(state: State<'_, AppAudio>, time: f64) {
    if let Ok(ctrl) = state.0.lock() { ctrl.seek(time); }
}

/// Tauri 命令：获取当前播放位置（秒）
#[tauri::command]
pub fn get_audio_position(state: State<'_, AppAudio>) -> f64 {
    if let Ok(ctrl) = state.0.lock() { ctrl.get_position() } else { 0.0 }
}

/// Tauri 命令：设置播放音量
#[tauri::command]
pub fn set_volume(state: State<'_, AppAudio>, vol: f32) {
    if let Ok(ctrl) = state.0.lock() { ctrl.set_volume(vol); }
}

#[tauri::command]
pub fn is_audio_playing(state: State<'_, AppAudio>) -> bool {
    if let Ok(ctrl) = state.0.lock() { ctrl.get_is_playing() } else { false }
}
