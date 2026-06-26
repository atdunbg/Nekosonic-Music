import { invoke } from '@tauri-apps/api/core';

/**
 * 私人 FM 相关 API
 */
export const FmApi = {
  /** 获取私人 FM 歌曲 */
  async personalFm(): Promise<string> {
    return invoke('personal_fm');
  },

  /** 切换私人 FM 模式 */
  async personalFmMode(query: { mode: string; subMode: string; limit: number }): Promise<string> {
    return invoke('personal_fm_mode', { query });
  },

  /** 私人 FM 不喜欢（扔进垃圾桶） */
  async fmTrash(id: number, time: number): Promise<void> {
    return invoke('fm_trash', { query: { id, time } });
  },
};
