import { invoke } from '@tauri-apps/api/core';

/**
 * 歌单相关 API
 */
export const PlaylistApi = {
  /** 获取用户歌单列表 */
  async userPlaylist(uid: number): Promise<string> {
    return invoke('user_playlist', { uid });
  },

  /** 获取歌单详情 */
  async getPlaylistDetail(id: number): Promise<string> {
    return invoke('get_playlist_detail', { id });
  },

  /** 获取歌单全部曲目 */
  async playlistTrackAll(id: number): Promise<string> {
    return invoke('playlist_track_all', { query: { id } });
  },

  /** 订阅/取消订阅歌单 */
  async playlistSubscribe(id: number, subscribe: boolean): Promise<void> {
    return invoke('playlist_subscribe', { query: { id, subscribe } });
  },

  /** 每日推荐歌单 */
  async recommendResource(): Promise<string> {
    return invoke('recommend_resource');
  },

  /** 每日推荐歌曲 */
  async recommendSongs(): Promise<string> {
    return invoke('recommend_songs');
  },
};
