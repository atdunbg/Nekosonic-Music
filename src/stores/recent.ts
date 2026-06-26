import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import type { Song } from '../types/song';

const RECENT_KEY = 'recent_local';
const MAX_RECENT = 200;

function loadRecentLocal(): Song[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* 忽略 */ }
  return [];
}

/**
 * 最近播放 store
 *
 * 管理最近播放过的歌曲列表，持久化到 localStorage。
 */
export const useRecentStore = defineStore('recent', () => {
  /** 最近播放的歌曲列表（按时间倒序） */
  const recentLocal = ref<Song[]>(loadRecentLocal());

  let persistTimer: ReturnType<typeof setTimeout> | undefined;

  /** 添加一首歌到最近播放列表（去重并放到最前） */
  function addRecent(song: Song) {
    recentLocal.value = recentLocal.value.filter(s => s.id !== song.id);
    recentLocal.value.unshift(song);
    if (recentLocal.value.length > MAX_RECENT) {
      recentLocal.value = recentLocal.value.slice(0, MAX_RECENT);
    }
  }

  watch(recentLocal, (val) => {
    clearTimeout(persistTimer);
    persistTimer = setTimeout(() => {
      localStorage.setItem(RECENT_KEY, JSON.stringify(val));
    }, 2000);
  }, { deep: true });

  return {
    recentLocal,
    addRecent,
  };
});
