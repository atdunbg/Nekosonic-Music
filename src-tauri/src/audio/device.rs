//! 音频输出设备管理
//!
//! 列举、查找、选择音频输出设备。当用户指定的设备不存在时回退到系统默认设备。

use cpal::traits::{DeviceTrait, HostTrait};

/// 获取系统默认输出设备的名称
pub fn get_system_default_device_name() -> Option<String> {
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
pub fn find_device_by_name(name: &str) -> Option<cpal::Device> {
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
pub fn get_output_device(selected: &Option<String>) -> cpal::Device {
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
