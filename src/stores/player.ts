import { defineStore } from 'pinia';
import { ref , watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { normalizeSong } from '../utils/song';
import { useSettingsStore } from './settings';
import { useUserStore } from './user';

export type PlayMode = 'loop' | 'shuffle' | 'repeat-one';

export interface Song {
  id: number;
  name: string;
  ar: { name: string }[];
  al: { picUrl: string };
  dt?: number;

  // 兼容不同接口返回的可选字段
  album?: { picUrl?: string };
  artists?: { name: string }[];
  duration?: number;   // 某些接口的时长字段（单位可能是秒）
}

const cacheProgress = ref(0);

// 监听 Tauri 事件（需要在适当位置初始化一次）
import { listen } from '@tauri-apps/api/event';

export function setupCacheProgressListener() {
  listen<number>('cache-progress', (event) => {
    cacheProgress.value = event.payload;
  });
}

// 在 store 定义外调用 setupCacheProgressListener()，或者在应用入口调用


function loadRecentLocal(): Song[] {
  try {
    const raw = localStorage.getItem('recent_local');
    if (raw) return JSON.parse(raw);
  } catch {}
  return [];
}

function loadLikedIdsFromStorage(): Set<number> {
  try {
    const raw = localStorage.getItem('liked_ids');
    if (raw) return new Set(JSON.parse(raw));
  } catch {}
  return new Set();
}

export const usePlayerStore = defineStore('player', () => {
  const currentSong = ref<Song | null>(null);
  const playing = ref(false);
  const currentTime = ref(0);
  const duration = ref(0);

  const queue = ref<Song[]>([]);
  const currentIndex = ref(-1);

  let tickInterval: ReturnType<typeof setInterval> | null = null;

  const recentLocal = ref<Song[]>(loadRecentLocal());
  const MAX_RECENT = 200;

  const likedIds = ref<Set<number>>(loadLikedIdsFromStorage());

  function isLiked(songId: number): boolean {
    return likedIds.value.has(songId);
  }

  async function loadLikedIds() {
    const userStore = useUserStore();
    if (!userStore.isLoggedIn) return;
    try {
      const json: string = await invoke('likelist', { uid: userStore.user!.userId });
      const data = JSON.parse(json);
      const ids: number[] = data.ids || data.data?.ids || [];
      likedIds.value = new Set(ids);
    } catch { /* 忽略 */ }
  }

  async function toggleLike(songId: number) {
    const wasLiked = likedIds.value.has(songId);
    const newLike = !wasLiked;
    try {
      await invoke('like_song', { query: { id: songId, like: newLike ? 'true' : 'false' } });
      if (newLike) {
        likedIds.value.add(songId);
      } else {
        likedIds.value.delete(songId);
      }
      likedIds.value = new Set(likedIds.value);
    } catch { /* 忽略 */ }
  }

  function addRecent(song: Song) {
    recentLocal.value = recentLocal.value.filter(s => s.id !== song.id);
    recentLocal.value.unshift(song);
    if (recentLocal.value.length > MAX_RECENT) {
      recentLocal.value = recentLocal.value.slice(0, MAX_RECENT);
    }
  }

  watch(recentLocal, (val) => {
    localStorage.setItem('recent_local', JSON.stringify(val));
  }, { deep: true });

  watch(likedIds, (val) => {
    localStorage.setItem('liked_ids', JSON.stringify([...val]));
  }, { deep: true });

  const isFmMode = ref(false);
  let fmNextCallback: (() => void) | null = null;

  function enableFmMode(onNext: () => void) {
    isFmMode.value = true;
    fmNextCallback = onNext;
  }

  function disableFmMode() {
    isFmMode.value = false;
    fmNextCallback = null;
  }

  // 播放私人漫游歌曲（清空队列，只播放这一首）
  async function playFmSong(song: any) {
    // 如果缺少时长，尝试从详情接口获取
    if (!song.dt || song.dt === 0) {
      try {
        const jsonStr: string = await invoke('get_song_detail', { id: String(song.id) });
        const data = JSON.parse(jsonStr);
        const full = data.songs?.[0];
        if (full) {
          song.dt = full.dt || 0;
          song.al = full.al || song.al;
          song.ar = full.ar || song.ar;
        }
      } catch (e) { /* 忽略 */ }
    }

    await invoke('stop_audio');
    queue.value = [];
    currentIndex.value = -1;
    playing.value = false;

    currentSong.value = song;
    try {
      const settings = useSettingsStore();
      const url: string = await invoke('get_song_url', { query: { id: Number(song.id), level: settings.audioQuality } });
      if (!url) throw new Error('无播放源');
      await invoke('play_audio', { url });
      playing.value = true;
      duration.value = (song.dt || 0) / 1000;
      currentTime.value = 0;
      startTick();
      addRecent(song);
    } catch (e) {
      console.error('FM播放失败', e);
      playing.value = false;
    }
  }

  // 播放指定歌曲（如果不在队列中则加入并切换）
  async function play(song: Song) {
    disableFmMode();

    const idx = queue.value.findIndex(s => s.id === song.id);
    if (idx === -1) {
      // 未在队列中，添加到队列并播放该位置
      queue.value.push(song);
      currentIndex.value = queue.value.length - 1;
    } else {
      currentIndex.value = idx;
    }
    await playCurrent();
  }

  async function playCurrent() {
    const song = queue.value[currentIndex.value];
    if (!song?.id) {
      console.error('无效的歌曲数据', song);
      return;
    }

    try {
      // 重置状态
      currentSong.value = song;
      playing.value = false;
      currentTime.value = 0;
      duration.value = (song.dt || 0) / 1000;

      const settings = useSettingsStore();
      const url: string = await invoke('get_song_url', { query: { id: Number(song.id), level: settings.audioQuality } });
      if (!url) {
        console.error('未获取到有效播放地址', song);
        return;
      }

      await invoke('play_audio', { url });
      playing.value = true;
      startTick();
      addRecent(song);
    } catch (e) {
      console.error('播放失败', e);
      playing.value = false;
    }
  }

  function startTick() {
    if (tickInterval) clearInterval(tickInterval);
    tickInterval = setInterval(() => {
      if (playing.value && duration.value > 0) {
        currentTime.value += 0.25;
        if (currentTime.value >= duration.value) {
          currentTime.value = duration.value;
          next(); // 自动下一首
        }
      }
    }, 250);
  }

  async function toggle() {
    if (playing.value) {
      await invoke('pause_audio');
      playing.value = false;
    } else {
      await invoke('resume_audio');
      playing.value = true;
    }
  }

  async function stop() {
    await invoke('stop_audio');
    playing.value = false;
    currentSong.value = null;
    currentTime.value = 0;
    if (tickInterval) clearInterval(tickInterval);
    disableFmMode(); // 停止时退出漫游
  }


  function prev() {
    if (isFmMode.value) return;
    if (queue.value.length === 0) return;
    currentIndex.value = (currentIndex.value - 1 + queue.value.length) % queue.value.length;
    playCurrent();
  }

  // 批量添加歌曲到队列并播放第一首（用于“播放全部”）
  async function playAll(songs: Song[]) {
    if (songs.length === 0) return;
    queue.value = [...songs];
    currentIndex.value = 0;
    await playCurrent();
  }

  function removeFromQueue(index: number) {
    if (index < 0 || index >= queue.value.length) return;
    const isCurrent = index === currentIndex.value;
    if (isCurrent) {
      // 如果移除的是当前正在播放的歌曲，先停止，然后调整索引
      stop();
      queue.value.splice(index, 1);
      // 如果队列变空，则重置
      if (queue.value.length === 0) {
        currentIndex.value = -1;
        return;
      }
      // 保持索引不变，但如果删的是最后一个，索引需要退一位
      if (currentIndex.value >= queue.value.length) {
        currentIndex.value = queue.value.length - 1;
      }
      // 不自动播放，等用户手动选择
    } else {
      queue.value.splice(index, 1);
      // 调整当前索引
      if (index < currentIndex.value) {
        currentIndex.value -= 1;
      }
    }
  }

  function clearQueue() {
    stop();
    queue.value = [];
    currentIndex.value = -1;
  }

  async function seek(time: number) {
    try {
      await invoke('seek_audio', { time });
      currentTime.value = time;
    } catch (e) {
      console.error('seek 失败', e);
    }
  }


  // 在 defineStore 内部添加
  const playMode = ref<PlayMode>('loop');

  function setPlayMode(mode: PlayMode) {
    playMode.value = mode;
  }

  // 重写 next() 以根据模式选择下一首
  function next() {
    if (isFmMode.value && fmNextCallback) {
      fmNextCallback();
      return;
    }

    if (queue.value.length === 0) return;
    let nextIndex: number;
    switch (playMode.value) {
      case 'repeat-one':
        // 单曲循环，不改变索引，只重新播放当前
        playCurrent();
        return;
      case 'shuffle':
        // 随机下一首，且不与当前重复（除非只剩一首）
        if (queue.value.length === 1) {
          nextIndex = 0;
        } else {
          do {
            nextIndex = Math.floor(Math.random() * queue.value.length);
          } while (nextIndex === currentIndex.value);
        }
        break;
      case 'loop':
      default:
        // 顺序循环
        nextIndex = (currentIndex.value + 1) % queue.value.length;
        break;
    }
    currentIndex.value = nextIndex;
    playCurrent();
  }

  const showRoamDrawer = ref(false);

  function openRoamDrawer() {
    showRoamDrawer.value = true;
  }

  function closeRoamDrawer() {
    showRoamDrawer.value = false;
  }

  function toggleRoamDrawer() {
    showRoamDrawer.value = !showRoamDrawer.value;
  }

  async function loadFirstFmSong() {
  try {
    const jsonStr: string = await invoke('personal_fm');
    const data = JSON.parse(jsonStr);
    const songs = data.data || data;
    if (songs && songs.length > 0) {
      const song = normalizeSong(songs[0]);
      enableFmMode(() => loadFirstFmSong()); // 下一首回调
      await playFmSong(song);
      return true;
    }
  } catch (e) {
    console.error(e);
  }
  return false;
}


// -------- FM 专属状态 --------
const fmSong = ref<any>(null);
const fmPlaying = ref(false);

async function loadFm() {
  try {
    const jsonStr: string = await invoke('personal_fm');
    const data = JSON.parse(jsonStr);
    const songs = data.data || data;
    if (songs && songs.length > 0) {
      const song = normalizeSong(songs[0]);
      fmSong.value = song;
      enableFmMode(nextFm);      // 设置下一首回调为 store 内的 nextFm
      await playFmSong(song);   // 使用 FM 专用播放方法
      fmPlaying.value = true;
      // showRoamDrawer.value = true; // 自动打开全屏抽屉
    }
  } catch (e) {
    console.error('FM加载失败', e);
  }
}

async function toggleFm() {
  if (!fmSong.value) return;
  if (fmPlaying.value) {
    // 当前 FM 正在播放，切换暂停/恢复
    await toggle(); // 全局暂停/播放
    fmPlaying.value = playing.value;
  } else {
    // FM 处于暂停状态，或者当前被其他歌曲打断
    if (currentSong.value?.id === fmSong.value.id) {
      // FM 歌曲还是当前歌曲，直接恢复
      await toggle();
      fmPlaying.value = playing.value;
    } else {
      // 当前播放的是其他歌曲，重新以 FM 模式播放 FM 歌曲
      enableFmMode(nextFm);
      await playFmSong(fmSong.value);
      fmPlaying.value = true;
    }
  }
}

async function nextFm() {
  await loadFm(); // 加载下一首 FM 歌曲
}

// 监听全局播放变化，若用户选择了非 FM 歌曲，自动退出 FM 状态
watch(currentSong, (newSong) => {
  if (isFmMode.value && newSong?.id !== fmSong.value?.id) {
    fmPlaying.value = false;
    // 注意：不调用 disableFmMode，因为可能只是临时切歌，但卡片需要知道 FM 已停止
    disableFmMode(); // 退出 FM 模式，让上一首按钮恢复
  }
});

watch(playing, (val) => {
  // 只有当前正在播放的是 FM 歌曲时，才同步 fmPlaying
  if (currentSong.value?.id === fmSong.value?.id) {
    fmPlaying.value = val;
  } else {
    fmPlaying.value = false;
  }
});


  return {
    currentSong,
    playing,
    currentTime,
    duration,
    queue,
    currentIndex,
    playMode,
    isFmMode,
    enableFmMode,
    disableFmMode,
    playFmSong,
    setPlayMode,
    play,
    playAll,
    toggle,
    stop,
    prev,
    next,
    seek,
    playCurrent,

    removeFromQueue,
    clearQueue,

    recentLocal,

    likedIds,
    isLiked,
    loadLikedIds,
    toggleLike,

    showRoamDrawer,
    openRoamDrawer,
    closeRoamDrawer,
    toggleRoamDrawer,
    loadFirstFmSong,

    fmSong,
    fmPlaying,
    loadFm,
    toggleFm,
    nextFm,
  };
});