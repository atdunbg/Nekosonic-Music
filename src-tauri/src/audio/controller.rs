//! 音频控制器
//!
//! 负责命令分发与播放生命周期管理。`AudioController` 是对外接口，
//! 通过通道向后台 `audio_thread` 发送命令；`audio_thread` 是核心事件循环，
//! 管理播放状态、设备热切换、播放结束检测等。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use symphonia::core::io::MediaSourceStream;
use tauri::{AppHandle, Emitter};

use crate::audio::buffer::{SharedBuffer, StreamingReader};
use crate::audio::device::{get_output_device, get_system_default_device_name};
use crate::audio::download::download_audio_streaming;
use crate::audio::output::{OutputContext, INITIAL_BUFFER_SIZE, start_playback};

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

/// 停止播放，取消解码并重置共享播放位置
fn stop_playback(output_ctx: &mut Option<OutputContext>, shared_position: &Arc<Mutex<f64>>) {
    if let Some(ref mut ctx) = output_ctx {
        ctx.playback.cancelled.store(true, Ordering::Relaxed);
        ctx.playback.playing.store(false, Ordering::Relaxed);
    }
    *output_ctx = None;
    *shared_position.lock().unwrap() = 0.0;
}

/// 根据当前播放源（本地文件或网络缓冲区）重建 MediaSourceStream
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

/// 设备切换时重启播放，保留当前播放位置与暂停状态
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
fn audio_thread(
    rx: Receiver<AudioCmd>,
    _current_url: Arc<Mutex<Option<String>>>,
    shared_position: Arc<Mutex<f64>>,
    is_playing: Arc<AtomicBool>,
    app_handle: AppHandle,
) {
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
