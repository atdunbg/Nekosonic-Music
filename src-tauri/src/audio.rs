use rodio::{Decoder, OutputStream, Sink, Source};
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;

// ---------- 命令 ----------
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

pub struct AudioController {
    tx: Sender<AudioCmd>,
    current_url: Arc<Mutex<Option<String>>>,
}

impl AudioController {
    pub fn new(app_handle: AppHandle) -> Self {
        let (tx, rx) = channel();
        let current_url = Arc::new(Mutex::new(None));
        let url_clone = current_url.clone();
        let ah_clone = app_handle.clone();
        thread::spawn(move || audio_thread(rx, url_clone, ah_clone));
        AudioController {
            tx,
            current_url,
        }
    }

    pub fn play_url(&self, url: &str) {
        *self.current_url.lock().unwrap() = Some(url.to_string());
        let _ = self.tx.send(AudioCmd::Play(url.to_string()));
    }
    pub fn play_local(&self, path: &str) {
        *self.current_url.lock().unwrap() = Some(path.to_string());
        let _ = self.tx.send(AudioCmd::PlayLocal(path.to_string()));
    }
    pub fn pause(&self) { let _ = self.tx.send(AudioCmd::Pause); }
    pub fn resume(&self) { let _ = self.tx.send(AudioCmd::Resume); }
    pub fn stop(&self) { let _ = self.tx.send(AudioCmd::Stop); }
    pub fn set_device(&self, device: Option<String>) {
        let _ = self.tx.send(AudioCmd::SetDevice(device));
    }
    pub fn seek(&self, time: f64) {
        let _ = self.tx.send(AudioCmd::Seek(time));
    }
    pub fn set_volume(&self, vol: f32) {
        let _ = self.tx.send(AudioCmd::SetVolume(vol));
    }
}

// ---------- 流式缓冲区 ----------

struct BufferState {
    bytes: Vec<u8>,
    done: bool,
    cancelled: bool,
}

struct SharedBuffer {
    state: Mutex<BufferState>,
    available: Condvar,
}

impl SharedBuffer {
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

    fn write_chunk(&self, chunk: &[u8]) {
        let mut state = self.state.lock().unwrap();
        state.bytes.extend_from_slice(chunk);
        self.available.notify_all();
    }

    fn mark_done(&self) {
        let mut state = self.state.lock().unwrap();
        state.done = true;
        self.available.notify_all();
    }

    fn cancel(&self) {
        let mut state = self.state.lock().unwrap();
        state.cancelled = true;
        self.available.notify_all();
    }

    fn len(&self) -> usize {
        self.state.lock().unwrap().bytes.len()
    }

    fn is_done(&self) -> bool {
        self.state.lock().unwrap().done
    }

    fn is_cancelled(&self) -> bool {
        self.state.lock().unwrap().cancelled
    }
}

struct StreamingReader {
    buffer: Arc<SharedBuffer>,
    pos: usize,
}

impl StreamingReader {
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
            let result = self.buffer.available.wait_timeout(state, Duration::from_millis(500)).unwrap();
            state = result.0;
        }
    }
}

impl Seek for StreamingReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::Current(offset) => self.pos as i64 + offset,
            SeekFrom::End(offset) => {
                let mut state = self.buffer.state.lock().unwrap();
                loop {
                    if state.done {
                        break state.bytes.len() as i64 + offset;
                    }
                    if state.cancelled {
                        return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "cancelled"));
                    }
                    let result = self.buffer.available.wait_timeout(state, Duration::from_millis(500)).unwrap();
                    state = result.0;
                }
            }
        };
        if new_pos < 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "seek before start"));
        }
        let mut state = self.buffer.state.lock().unwrap();
        loop {
            if new_pos as usize <= state.bytes.len() {
                self.pos = new_pos as usize;
                return Ok(self.pos as u64);
            }
            if state.done {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "seek past end"));
            }
            if state.cancelled {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "cancelled"));
            }
            let result = self.buffer.available.wait_timeout(state, Duration::from_millis(500)).unwrap();
            state = result.0;
        }
    }
}

fn download_audio_streaming(
    url: &str,
    buffer: &SharedBuffer,
    app_handle: &AppHandle,
) -> Result<(), String> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| format!("下载失败: {}", e))?;

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
        let read_size = reader.read(&mut chunk)
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

const INITIAL_BUFFER_SIZE: usize = 65536;

// ---------- 音频线程 ----------
fn audio_thread(rx: Receiver<AudioCmd>, current_url: Arc<Mutex<Option<String>>>, app_handle: AppHandle) {
    let mut selected_device: Option<String> = None;
    let mut output = create_output(&selected_device);
    let mut last_default_name = get_system_default_device_name();

    let mut current_volume: f32 = 1.0;
    if let Some(ref sink) = output.sink {
        sink.set_volume(current_volume);
    }

    let mut current_audio_buffer: Option<Arc<SharedBuffer>> = None;
    let mut audio_active = false;
    let mut audio_paused = false;
    let mut manual_stop = false;

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(cmd) => {
                match cmd {
                    AudioCmd::Play(url) => {
                        audio_active = false;
                        audio_paused = false;
                        manual_stop = false;
                        if let Some(ref buf) = current_audio_buffer {
                            buf.cancel();
                        }
                        if let Some(ref sink) = output.sink { sink.stop(); }
                        output = create_output(&selected_device);
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(current_volume);

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
                                std::thread::sleep(Duration::from_millis(50));
                            }

                            if buffer.is_cancelled() || buffer.len() == 0 {
                                current_audio_buffer = None;
                                continue;
                            }

                            let reader = StreamingReader::new(buffer.clone());
                            match Decoder::new(reader) {
                                Ok(source) => {
                                    sink.append(source);
                                    sink.play();
                                    audio_active = true;
                                    let _ = app_handle.emit("audio-started", ());
                                }
                                Err(e) => {
                                    eprintln!("[audio] 流式解码失败: {}, 等待完整下载后重试", e);
                                    loop {
                                        if buffer.is_done() || buffer.is_cancelled() {
                                            break;
                                        }
                                        std::thread::sleep(Duration::from_millis(100));
                                    }
                                    if buffer.is_cancelled() || buffer.len() == 0 {
                                        current_audio_buffer = None;
                                        continue;
                                    }
                                    let buf = current_audio_buffer.as_ref().unwrap().clone();
                                    let reader2 = StreamingReader::new(buf);
                                    match Decoder::new(reader2) {
                                        Ok(source) => {
                                            sink.append(source);
                                            sink.play();
                                            audio_active = true;
                                            let _ = app_handle.emit("audio-started", ());
                                        }
                                        Err(e2) => {
                                            eprintln!("[audio] 完整下载后解码也失败: {}", e2);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    AudioCmd::PlayLocal(path) => {
                        audio_active = false;
                        audio_paused = false;
                        manual_stop = false;
                        if let Some(ref buf) = current_audio_buffer {
                            buf.cancel();
                        }
                        if let Some(ref sink) = output.sink { sink.stop(); }
                        output = create_output(&selected_device);
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(current_volume);

                            match std::fs::read(&path) {
                                Ok(bytes) => {
                                    let buffer = Arc::new(SharedBuffer::new());
                                    buffer.write_chunk(&bytes);
                                    buffer.mark_done();
                                    current_audio_buffer = Some(buffer.clone());

                                    let reader = StreamingReader::new(buffer);
                                    match Decoder::new(reader) {
                                        Ok(source) => {
                                            sink.append(source);
                                            sink.play();
                                            audio_active = true;
                                            let _ = app_handle.emit("audio-started", ());
                                        }
                                        Err(e) => eprintln!("[audio] 本地播放失败: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("[audio] 读取本地文件失败: {}", e),
                            }
                        }
                    }

                    AudioCmd::Pause => {
                        audio_paused = true;
                        if let Some(ref sink) = output.sink { sink.pause(); }
                    }
                    AudioCmd::Resume => {
                        audio_paused = false;
                        if let Some(ref sink) = output.sink { sink.play(); }
                    }
                    AudioCmd::Stop => {
                        audio_active = false;
                        audio_paused = false;
                        manual_stop = true;
                        if let Some(ref buf) = current_audio_buffer {
                            buf.cancel();
                        }
                        if let Some(ref sink) = output.sink { sink.stop(); }
                    }

                    AudioCmd::Seek(time) => {
                        if let Some(ref sink) = output.sink {
                            let seek_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                sink.try_seek(Duration::from_secs_f64(time))
                            }));

                            match seek_res {
                                Ok(Ok(_)) => { /* 成功 */ }
                                Ok(Err(e)) => {
                                    eprintln!("[audio] try_seek 失败: {:?}, 回退重建解码", e);
                                    if let Some(ref buffer) = current_audio_buffer {
                                        sink.stop();
                                        sink.clear();
                                        let reader = StreamingReader::new(buffer.clone());
                                        match Decoder::new(reader) {
                                            Ok(source) => {
                                                let source = source.skip_duration(Duration::from_secs_f64(time));
                                                sink.append(source);
                                                sink.play();
                                            }
                                            Err(e) => eprintln!("[audio] seek 解码失败: {}", e),
                                        }
                                    }
                                }
                                Err(_) => {
                                    eprintln!("[audio] try_seek 崩溃，回退重建解码");
                                    if let Some(ref buffer) = current_audio_buffer {
                                        sink.stop();
                                        sink.clear();
                                        let reader = StreamingReader::new(buffer.clone());
                                        match Decoder::new(reader) {
                                            Ok(source) => {
                                                let source = source.skip_duration(Duration::from_secs_f64(time));
                                                sink.append(source);
                                                sink.play();
                                            }
                                            Err(e) => eprintln!("[audio] seek 解码失败: {}", e),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    AudioCmd::SetVolume(vol) => {
                        current_volume = vol;
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(vol);
                        }
                    }

                    AudioCmd::SetDevice(dev) => {
                        selected_device = dev;
                        output = create_output(&selected_device);
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(current_volume);
                            if current_url.lock().unwrap().is_some() {
                                if let Some(ref buffer) = current_audio_buffer {
                                    let reader = StreamingReader::new(buffer.clone());
                                    match Decoder::new(reader) {
                                        Ok(source) => {
                                            sink.append(source);
                                            sink.play();
                                        }
                                        Err(e) => eprintln!("[audio] 设备切换解码失败: {}", e),
                                    }
                                }
                            }
                        }
                        if selected_device.is_none() {
                            last_default_name = get_system_default_device_name();
                        }
                    }
                }
            }

            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if audio_active && !audio_paused {
                    if let Some(ref sink) = output.sink {
                        if sink.empty() {
                            audio_active = false;
                            if !manual_stop {
                                let _ = app_handle.emit("audio-ended", ());
                            }
                        }
                    }
                }
                if selected_device.is_none() {
                    let current_default = get_system_default_device_name();
                    if current_default != last_default_name {
                        println!("[audio] 系统默认设备变化: {:?} -> {:?}", last_default_name, current_default);
                        last_default_name = current_default;
                        output = create_output(&selected_device);
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(current_volume);
                            if let Some(ref buffer) = current_audio_buffer {
                                let reader = StreamingReader::new(buffer.clone());
                                let _ = Decoder::new(reader).map(|source| {
                                    sink.append(source);
                                    sink.play();
                                });
                            }
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
}

// ---------- 其余函数保持不变 ----------

fn get_system_default_device_name() -> Option<String> {
    rodio::cpal::default_host()
        .default_output_device()
        .and_then(|d| d.name().ok())
}

pub fn list_output_devices() -> Vec<String> {
    let host = rodio::cpal::default_host();
    if let Ok(devices) = host.output_devices() {
        let mut names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();
        names.sort();
        names.dedup();
        names
    } else {
        vec![]
    }
}

fn find_device_by_name(name: &str) -> Option<rodio::cpal::Device> {
    let host = rodio::cpal::default_host();
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

struct Output {
    _stream: OutputStream,
    sink: Option<Sink>,
}

fn create_output(selected_device: &Option<String>) -> Output {
    match selected_device {
        Some(dev_name) => {
            if let Some(dev) = find_device_by_name(dev_name) {
                println!("[audio] 使用指定设备: {}", dev_name);
                match OutputStream::try_from_device(&dev) {
                    Ok((stream, handle)) => {
                        match Sink::try_new(&handle) {
                            Ok(sink) => Output { _stream: stream, sink: Some(sink) },
                            Err(e) => {
                                eprintln!("[audio] Sink 创建失败: {}", e);
                                Output { _stream: stream, sink: None }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[audio] 指定设备无效，回退默认: {}", e);
                        create_default_output()
                    }
                }
            } else {
                eprintln!("[audio] 未找到设备 `{}`，回退默认", dev_name);
                create_default_output()
            }
        }
        None => {
            println!("[audio] 跟随系统默认设备");
            create_default_output()
        }
    }
}

fn create_default_output() -> Output {
    match OutputStream::try_default() {
        Ok((stream, handle)) => {
            match Sink::try_new(&handle) {
                Ok(sink) => Output { _stream: stream, sink: Some(sink) },
                Err(e) => {
                    eprintln!("[audio] 默认 Sink 失败: {}", e);
                    Output { _stream: stream, sink: None }
                }
            }
        }
        Err(e) => panic!("无法创建默认音频输出: {}", e),
    }
}

// ===================== Tauri 命令 =====================
use tauri::State;
use std::sync::Mutex as StdMutex;

pub struct AppAudio(pub StdMutex<AudioController>);

#[tauri::command]
pub fn play_audio(state: State<'_, AppAudio>, url: String) -> Result<(), String> {
    let ctrl = state.0.lock().map_err(|e| e.to_string())?;
    ctrl.play_url(&url);
    Ok(())
}

#[tauri::command]
pub fn play_local_audio(state: State<'_, AppAudio>, path: String) -> Result<(), String> {
    let ctrl = state.0.lock().map_err(|e| e.to_string())?;
    ctrl.play_local(&path);
    Ok(())
}

#[tauri::command]
pub fn pause_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.pause(); }
}

#[tauri::command]
pub fn resume_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.resume(); }
}

#[tauri::command]
pub fn stop_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.stop(); }
}

#[tauri::command]
pub fn get_output_devices() -> Vec<String> {
    list_output_devices()
}

#[tauri::command]
pub fn set_output_device(state: State<'_, AppAudio>, device: Option<String>) {
    if let Ok(ctrl) = state.0.lock() {
        ctrl.set_device(device);
    }
}

#[tauri::command]
pub fn seek_audio(state: State<'_, AppAudio>, time: f64) {
    if let Ok(ctrl) = state.0.lock() {
        ctrl.seek(time);
    }
}

#[tauri::command]
pub fn set_volume(state: State<'_, AppAudio>, vol: f32) {
    if let Ok(ctrl) = state.0.lock() {
        ctrl.set_volume(vol);
    }
}
