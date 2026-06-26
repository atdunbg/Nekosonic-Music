//! 音频模块
//!
//! 负责音频的解码、播放、设备管理与对外 Tauri 命令。
//!
//! 模块结构：
//! - [`buffer`]: 流式缓冲区与读取器
//! - [`download`]: 网络音频流式下载
//! - [`decoder`]: symphonia 解码 + 重采样 + 声道重混
//! - [`output`]: cpal 输出流与播放状态
//! - [`device`]: 输出设备列举与选择
//! - [`controller`]: 命令分发与播放生命周期
//! - [`commands`]: 对外 Tauri 命令

pub mod buffer;
pub mod commands;
pub mod controller;
pub mod decoder;
pub mod device;
pub mod download;
pub mod output;

// 对外重导出，保持与旧 `audio.rs` 相同的公开 API
pub use commands::AppAudio;
pub use controller::AudioController;
