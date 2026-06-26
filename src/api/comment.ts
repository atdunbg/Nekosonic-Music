import { invoke } from '@tauri-apps/api/core';

/**
 * 评论相关 API
 */
export const CommentApi = {
  /** 获取热门评论 */
  async commentHot(query: { type: number; id: number; limit: number; offset: number }): Promise<string> {
    return invoke('comment_hot', { query });
  },

  /** 点赞/取消点赞评论 */
  async commentLike(query: { t: number; type: number; id: number; cid: number }): Promise<void> {
    return invoke('comment_like', { query });
  },
};
