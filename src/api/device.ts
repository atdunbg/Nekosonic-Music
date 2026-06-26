import { invoke } from '@tauri-apps/api/core';

/**
 * 音频输出设备 API
 */
export const DeviceApi = {
  /** 获取所有输出设备 */
  async getOutputDevices(): Promise<string[]> {
    return invoke('get_output_devices');
  },

  /** 设置输出设备（null 表示默认） */
  async setOutputDevice(device: string | null): Promise<void> {
    return invoke('set_output_device', { device });
  },
};
