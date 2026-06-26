import { invoke } from '@tauri-apps/api/core';

/**
 * 登录相关 API
 */
export const LoginApi = {
  /** 获取登录状态 */
  async getLoginStatus(): Promise<string> {
    return invoke('get_login_status');
  },

  /** 退出登录 */
  async logout(): Promise<void> {
    return invoke('logout');
  },

  /** 获取二维码登录 key */
  async getQrKey(): Promise<string> {
    return invoke('get_qr_key');
  },

  /** 检查二维码扫描状态 */
  async checkQrStatus(key: string): Promise<string> {
    return invoke('check_qr_status', { query: { key } });
  },
};
