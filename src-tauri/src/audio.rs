use rodio::{Decoder, OutputStream, Sink, Source};
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use std::io::Cursor;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;

// ---------- 命令 ----------
enum AudioCmd {
    Play(String),
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
    let ah_clone = app_handle.clone();   // 克隆一个用于闭包
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

use std::io::Read;

fn download_audio_with_progress(
    url: &str,
    app_handle: &AppHandle,
) -> Result<Vec<u8>, String> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| format!("下载失败: {}", e))?;

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut buffer = Vec::new();
    let mut reader = resp;

    loop {
        let mut chunk = [0u8; 8192];
        let read_size = reader.read(&mut chunk)
            .map_err(|e| format!("读取失败: {}", e))?;
        if read_size == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_size]);
        downloaded += read_size as u64;

        // 发送进度事件给前端（每 8192 字节发一次，不必太频繁）
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0 // 未知大小时为 0
        };
        let _ = app_handle.emit("cache-progress", progress);
    }

    // 下载完成，确保进度为 100
    let _ = app_handle.emit("cache-progress", 100f64);
    Ok(buffer)
}

// ---------- 音频线程 ----------
fn audio_thread(rx: Receiver<AudioCmd>, current_url: Arc<Mutex<Option<String>>>, app_handle: AppHandle) {
    let mut selected_device: Option<String> = None;
    let mut output = create_output(&selected_device);
    let mut last_default_name = get_system_default_device_name();

    let mut current_volume: f32 = 1.0;
    if let Some(ref sink) = output.sink {
        sink.set_volume(current_volume);
    }

    let mut current_audio_data: Option<Vec<u8>> = None;  // 缓存原始音频字节

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(cmd) => {
                match cmd {
                    AudioCmd::Play(url) => {
                        // 停止旧播放并重建干净输出
                        if let Some(ref sink) = output.sink {
                            sink.stop();
                        }
                        output = create_output(&selected_device);
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(current_volume);

                            match download_audio_with_progress(&url, &app_handle) {
                                Ok(bytes) => {
                                    current_audio_data = Some(bytes.clone());
                                    let play_res = play_bytes(&bytes, sink);
                                    if let Err(e) = play_res {
                                        eprintln!("[audio] 播放失败: {}", e);
                                    }
                                }
                                Err(e) => eprintln!("[audio] 下载失败: {}", e),
                            }
                        }
                    }

                    AudioCmd::Pause => {
                        if let Some(ref sink) = output.sink { sink.pause(); }
                    }
                    AudioCmd::Resume => {
                        if let Some(ref sink) = output.sink { sink.play(); }
                    }
                    AudioCmd::Stop => {
                        if let Some(ref sink) = output.sink { sink.stop(); }
                    }

                    AudioCmd::Seek(time) => {
                        if let Some(ref sink) = output.sink {
                            // 优先尝试高效的 sink.try_seek（毫秒级）
                            let seek_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                sink.try_seek(Duration::from_secs_f64(time))
                            }));

                            match seek_res {
                                Ok(Ok(_)) => { /* 成功 */ }
                                Ok(Err(e)) => {
                                    eprintln!("[audio] try_seek 失败: {:?}, 回退重建解码", e);
                                    // 回退方案：重新解码并从目标时间开始
                                    if let Some(ref bytes) = current_audio_data {
                                        sink.stop();
                                        sink.clear();
                                        let _ = play_bytes_with_seek(bytes, sink, time);
                                    }
                                }
                                Err(_) => {
                                    eprintln!("[audio] try_seek 崩溃，回退重建解码");
                                    if let Some(ref bytes) = current_audio_data {
                                        sink.stop();
                                        sink.clear();
                                        let _ = play_bytes_with_seek(bytes, sink, time);
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
                            // 如果正在播放，恢复播放
                            if current_url.lock().unwrap().is_some() {
                                if let Some(ref bytes) = current_audio_data {
                                    let _ = play_bytes(bytes, sink);
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
                // 跟随系统默认设备变化
                if selected_device.is_none() {
                    let current_default = get_system_default_device_name();
                    if current_default != last_default_name {
                        println!("[audio] 系统默认设备变化: {:?} -> {:?}", last_default_name, current_default);
                        last_default_name = current_default;
                        output = create_output(&selected_device);
                        if let Some(ref sink) = output.sink {
                            sink.set_volume(current_volume);
                            if let Some(ref bytes) = current_audio_data {
                                let _ = play_bytes(bytes, sink);
                            }
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
}

// ---------- 播放辅助函数 ----------

/// 直接播放字节数据
fn play_bytes(bytes: &[u8], sink: &Sink) -> Result<(), String> {
    let cursor = Cursor::new(bytes.to_vec());
    let source = Decoder::new(cursor).map_err(|e| format!("解码失败: {}", e))?;
    sink.append(source);
    sink.play();
    Ok(())
}

/// 播放字节数据并跳过指定秒数（用于 seek 回退）
fn play_bytes_with_seek(bytes: &[u8], sink: &Sink, seek_secs: f64) -> Result<(), String> {
    let cursor = Cursor::new(bytes.to_vec());
    let source = Decoder::new(cursor).map_err(|e| format!("解码失败: {}", e))?;
    let source = source.skip_duration(Duration::from_secs_f64(seek_secs));
    sink.append(source);
    sink.play();
    Ok(())
}

// ---------- 其余函数保持不变（获取设备、创建输出等） ----------

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