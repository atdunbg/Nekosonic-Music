import { invoke } from '@tauri-apps/api/core';

/**
 * 歌手相关 API
 */
export const ArtistApi = {
  /** 获取歌手详情 */
  async artistDetail(id: number): Promise<string> {
    return invoke('artist_detail', { id });
  },

  /** 获取歌手歌曲 */
  async artistSongs(query: { id: number; order: string; limit: number; offset: number }): Promise<string> {
    return invoke('artist_songs', { query });
  },

  /** 获取歌手专辑 */
  async artistAlbum(id: number, limit: number, offset: number): Promise<string> {
    return invoke('artist_album', { id, limit, offset });
  },

  /** 获取歌手描述 */
  async artistDesc(id: number): Promise<string> {
    return invoke('artist_desc', { id });
  },

  /** 关注/取消关注歌手 */
  async artistSub(id: number, sub: boolean): Promise<string> {
    return invoke('artist_sub', { query: { id, sub } });
  },

  /** 已关注的歌手列表 */
  async artistSublist(limit = 100, offset = 0): Promise<string> {
    return invoke('artist_sublist', { query: { limit, offset } });
  },
};
