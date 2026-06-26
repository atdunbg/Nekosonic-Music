import { invoke } from '@tauri-apps/api/core';

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

  /** 云搜索 */
  async cloudsearch(query: { keyword: string; searchType: number; limit: number }): Promise<string> {
    return invoke('cloudsearch', { query });
  },
};
