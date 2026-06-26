import { invoke } from '@tauri-apps/api/core';

/**
 * 歌曲相关 API
 */
export const SongApi = {
  /** 获取歌曲详情 */
  async getSongDetail(id: string): Promise<string> {
    return invoke('get_song_detail', { id });
  },

  /** 获取歌曲播放 URL */
  async getSongUrl(query: { id: number; level: string; fm_mode?: boolean }): Promise<string> {
    return invoke('get_song_url', { query });
  },

  /** 获取歌词 */
  async getLyric(id: number): Promise<string> {
    return invoke('get_lyric', { id });
  },

  /** 获取喜欢列表 */
  async likelist(uid: number): Promise<string> {
    return invoke('likelist', { uid });
  },

  /** 喜欢/取消喜欢歌曲 */
  async likeSong(id: number, like: boolean): Promise<void> {
    return invoke('like_song', { query: { id, like: like ? 'true' : 'false' } });
  },

  /** 上报听歌记录（scrobble） */
  async scrobble(query: { id: number; sourceid: string; time: number; alg?: string; source?: string; bitrate?: number }): Promise<void> {
    return invoke('scrobble', { query });
  },
};
