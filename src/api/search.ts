import { invoke } from '@tauri-apps/api/core';
import type { MusicSourceConfig } from '../stores/settings';

/**
 * 搜索相关 API
 */
export const SearchApi = {
  /** 搜索建议 */
  async searchSuggest(keyword: string): Promise<string> {
    return invoke('search_suggest', { query: { keyword } });
  },

  /** 热门搜索 */
  async getHotSearch(): Promise<string> {
    return invoke('get_hot_search');
  },

  /** 云搜索（网易云） */
  async cloudsearch(query: { keyword: string; searchType: number; limit: number }): Promise<string> {
    return invoke('cloudsearch', { query });
  },

  /** 多源聚合搜索：并发请求所有启用音源，合并结果返回
   *  返回 JSON: ExternalSong[]（与后端 ExternalSong 结构对齐）
   *  用于「网易云搜不到的歌，从其他音源补充搜索结果」场景
   */
  async searchSongsMulti(query: {
    keyword: string;
    sources: MusicSourceConfig[];
    limit?: number;
  }): Promise<string> {
    return invoke('search_songs_multi', { query });
  },
};
