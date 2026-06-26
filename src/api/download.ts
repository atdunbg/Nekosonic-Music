import { invoke } from '@tauri-apps/api/core';

/**
 * 下载与本地音乐 API
 */
export const DownloadApi = {
  /** 下载歌曲 */
  async downloadSong(query: {
    id: number;
    name: string;
    artist: string;
    album: string | null;
    duration: number | null;
    coverUrl: string | null;
    level: string;
    downloadPath: string | null;
  }): Promise<void> {
    return invoke('download_song', { query });
  },

  /** 列出已下载的歌曲 */
  async listLocalSongs(downloadPath: string | null): Promise<any[]> {
    return invoke('list_local_songs', { downloadPath });
  },

  /** 扫描本地文件夹 */
  async scanLocalFolders(paths: string[]): Promise<any[]> {
    return invoke('scan_local_folders', { paths });
  },

  /** 删除本地歌曲文件 */
  async deleteLocalSong(query: { id: number; filename: string; downloadPath: string | null }): Promise<void> {
    return invoke('delete_local_song', { query });
  },

  /** 获取默认下载路径 */
  async getDefaultDownloadPath(): Promise<string> {
    return invoke('get_default_download_path');
  },
};
