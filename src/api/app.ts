import { invoke } from '@tauri-apps/api/core';

/**
 * 应用级 API
 */
export const AppApi = {
  /** 退出应用 */
  exitApp(): Promise<void> {
    return invoke('exit_app');
  },

  /** 读取图片为 data URL */
  async readImageAsDataUrl(path: string): Promise<string> {
    return invoke('read_image_as_data_url', { path });
  },

  /** 在文件管理器中显示文件 */
  async showItemInFolder(path: string): Promise<void> {
    return invoke('show_item_in_folder', { path });
  },
};
