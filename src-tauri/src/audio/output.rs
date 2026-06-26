//! 音频输出
//!
//! 基于 cpal 构建音频输出流，将环形缓冲区中的样本推送到设备。
//! 同时定义播放状态 `PlaybackState` 与输出上下文 `OutputContext`。

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{SampleRate, Stream, StreamConfig};
use ringbuf::traits::Consumer;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use symphonia::core::io::MediaSourceStream;

use crate::audio::buffer::{RingConsumer, create_ring_buffer};
use crate::audio::decoder::decode_to_ring;

/// 初始缓冲区大小，达到此字节数后才开始播放
pub const INITIAL_BUFFER_SIZE: usize = 65536;
/// 环形缓冲区容量（采样数），约 4 秒的 48kHz 立体声数据
pub const RING_BUFFER_SAMPLES: usize = 48000 * 4;

/// 播放状态，记录当前播放的运行时信息
pub struct PlaybackState {
    pub playing: Arc<AtomicBool>,
    pub cancelled: Arc<AtomicBool>,
    pub decode_done: Arc<AtomicBool>,
    pub buffer_exhausted: Arc<AtomicBool>,
    pub volume: Arc<Mutex<f32>>,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples_played: Arc<AtomicU64>,
    pub start_time: f64,
}

impl PlaybackState {
    /// 根据已播放采样数计算当前播放位置（秒）
    pub fn position(&self) -> f64 {
        let samples = self.samples_played.load(Ordering::Relaxed) as f64;
        self.start_time + samples / (self.sample_rate as f64 * self.channels as f64)
    }
}

/// 输出上下文，持有音频输出流和解码线程的句柄
pub struct OutputContext {
    pub _stream: Stream,
    pub _decode_thread: thread::JoinHandle<()>,
    pub playback: PlaybackState,
}

/// 启动音频播放，创建解码线程和 cpal 输出流
pub fn start_playback(
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

    let (producer, consumer) = create_ring_buffer(RING_BUFFER_SAMPLES);

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

    let stream = build_cpal_stream(
        device,
        sr,
        ch,
        sample_format,
        consumer,
        volume.clone(),
        playing.clone(),
        samples_played.clone(),
        decode_done.clone(),
        buffer_exhausted.clone(),
    )?;
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
#[allow(clippy::too_many_arguments)]
fn build_cpal_stream(
    device: &cpal::Device,
    sample_rate: u32,
    channels: u16,
    sample_format: cpal::SampleFormat,
    mut consumer: RingConsumer,
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
