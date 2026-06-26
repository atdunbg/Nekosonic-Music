import { invoke } from '@tauri-apps/api/core';

/**
 * 云盘相关 API
 */
export const CloudApi = {
  /** 获取云盘歌曲列表 */
  async userCloud(limit = 30, offset = 0): Promise<string> {
    return invoke('user_cloud', { limit, offset });
  },

  /** 删除云盘歌曲 */
  async userCloudDel(id: number): Promise<string> {
    return invoke('user_cloud_del', { id });
  },

  /** 上传歌曲到云盘 */
  async cloudUpload(filePath: string): Promise<string> {
    return invoke('cloud_upload', { filePath });
  },
};
