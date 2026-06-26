import { invoke } from '@tauri-apps/api/core';

/**
 * 推荐相关 API
 */
export const RecApi = {
  /** 个性化推荐歌单（无需登录） */
  async personalized(limit: number = 30): Promise<string> {
    return invoke('personalized', { limit });
  },

  /** 推荐新歌 */
  async personalizedNewsong(limit: number = 10): Promise<string> {
    return invoke('personalized_newsong', { limit });
  },

  /** 热门歌手 */
  async topArtists(limit: number = 30, offset: number = 0): Promise<string> {
    return invoke('top_artists', { limit, offset });
  },

  /** 新歌速递，type: 全部:0 / 华语:7 / 欧美:96 / 韩国:16 / 日本:8 */
  async topSong(areaType: number = 0): Promise<string> {
    return invoke('top_song', { areaType });
  },

  /** 最新专辑（新碟上架） */
  async albumNewest(): Promise<string> {
    return invoke('album_newest');
  },
};
