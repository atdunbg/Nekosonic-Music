import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useUserStore } from './user';
import { MusicApi } from '../api';

/**
 * 喜欢列表 store
 *
 * 管理用户喜欢的歌曲 ID 集合，支持加载、查询、切换喜欢状态。
 */
export const useLikedStore = defineStore('liked', () => {
  /** 已喜欢的歌曲 ID 集合 */
  const likedIds = ref<Set<number>>(new Set());

  /** 判断指定歌曲是否已喜欢 */
  function isLiked(songId: number): boolean {
    return likedIds.value.has(songId);
  }

  /** 加载当前用户的喜欢列表 */
  async function loadLikedIds() {
    const userStore = useUserStore();
    if (!userStore.isLoggedIn) return;
    try {
      const json = await MusicApi.likelist(userStore.user!.userId);
      const data = JSON.parse(json);
      const ids: number[] = data.ids || data.data?.ids || [];
      likedIds.value = new Set(ids);
    } catch (e) {
      console.error('加载喜欢列表失败', e);
    }
  }

  /** 切换指定歌曲的喜欢状态 */
  async function toggleLike(songId: number) {
    const wasLiked = likedIds.value.has(songId);
    const newLike = !wasLiked;
    try {
      await MusicApi.likeSong(songId, newLike);
      if (newLike) {
        likedIds.value.add(songId);
      } else {
        likedIds.value.delete(songId);
      }
      likedIds.value = new Set(likedIds.value);
    } catch { /* 忽略 */ }
  }

  return {
    likedIds,
    isLiked,
    loadLikedIds,
    toggleLike,
  };
});
