import { invoke } from '@tauri-apps/api/core';

/**
 * 专辑相关 API
 */
export const AlbumApi = {
  /** 获取专辑详情 */
  async albumDetail(id: number): Promise<string> {
    return invoke('album_detail', { id });
  },
};
