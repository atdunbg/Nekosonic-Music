import { defineStore } from 'pinia';
import { ref, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { normalizeSong, type Song } from '../utils/song';
import { useSettingsStore } from './settings';
import { useUserStore } from './user';
import { showToast } from '../composables/useToast';

export type PlayMode = 'loop' | 'shuffle' | 'repeat-one';
export type { Song };

import { listen, emit } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

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

  const settings = useSettingsStore();
  const volume = ref(settings.volume);

  watch(volume, (val) => { settings.volume = val; });

  let tickInterval: ReturnType<typeof setInterval> | null = null;
  function setTickInterval(v: ReturnType<typeof setInterval> | null) { _tickInterval = v; tickInterval = v; }

  const recentLocal = ref<Song[]>(loadRecentLocal());
  const MAX_RECENT = 200;

  const likedIds = ref<Set<number>>(loadLikedIdsFromStorage());

  function emitPlaybackState() {
    const song = currentSong.value;
    const status = playing.value ? 'playing' : (song ? 'paused' : 'stopped');
    emit('playback-state', {
      status,
      title: song?.name || '',
      album: song?.al?.name || '',
      artists: song?.ar?.map(a => a.name) || [],
      coverUrl: song?.al?.picUrl || '',
      durationUs: (song?.dt || 0) * 1000,
      positionUs: Math.round(currentTime.value * 1_000_000),
      volume: volume.value / 100,
    });
  }

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
  const fmQueue: Song[] = [];
  let fmNextCallback: (() => void) | null = null;

  let lastScrobbleId: number | null = null;
  let lastScrobbleStartTime: number = 0;

  function reportScrobble() {
    const song = currentSong.value;
    if (!song || song.localPath || song.id == null) {
      lastScrobbleId = null;
      return;
    }
    if (lastScrobbleId === song.id && lastScrobbleStartTime > 0) {
      const playedSec = Math.round((Date.now() - lastScrobbleStartTime) / 1000);
      if (playedSec > 5) {
        invoke('scrobble', {
          query: {
            id: song.id,
            sourceid: isFmMode.value ? String(song.id) : '',
            time: playedSec,
          },
        }).catch(() => {});
      }
    }
    lastScrobbleId = song.id;
    lastScrobbleStartTime = Date.now();
  }

  function enableFmMode(onNext: () => void) {
    isFmMode.value = true;
    fmNextCallback = onNext;
  }

  function disableFmMode() {
    isFmMode.value = false;
    fmNextCallback = null;
    fmQueue.length = 0;
  }

  async function fetchFmBatch(): Promise<Song[]> {
    const jsonStr: string = await invoke('personal_fm');
    const data = JSON.parse(jsonStr);
    const raw = data.data || data;
    if (!Array.isArray(raw) || raw.length === 0) return [];
    return raw.map((s: any) => normalizeSong(s));
  }

  let fmVipSkipCount = 0;
  const MAX_FM_VIP_SKIP = 10;

  async function playFmSong(song: Song) {
    if (tickInterval) { clearInterval(tickInterval); setTickInterval(null); }
    reportScrobble();
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

    fmSong.value = song;
    currentSong.value = song;
    try {
      const jsonStr: string = await invoke('get_song_url', { query: { id: Number(song.id), level: settings.audioQuality, fm_mode: true } });
      const data = JSON.parse(jsonStr);
      const url: string | undefined = data.url;
      if (!url) throw new Error('无播放源');

      if (data.freeTrialInfo) {
        console.warn('FM VIP 试听歌曲，自动跳过', song.name);
        showToast(`${song.name} 为 VIP 试听，已跳过`, 'info');
        fmVipSkipCount++;
        if (fmVipSkipCount >= MAX_FM_VIP_SKIP) {
          console.warn('FM 连续跳过 VIP 歌曲过多，停止');
          fmVipSkipCount = 0;
          disableFmMode();
          return;
        }
        if (fmNextCallback) {
          fmNextCallback();
        } else {
          disableFmMode();
        }
        return;
      }

      fmVipSkipCount = 0;
      await invoke('play_audio', { url });
      await waitForAudioStart();
      playing.value = true;
      duration.value = (song.dt || 0) / 1000;
      currentTime.value = 0;
      startTick();
      addRecent(song);
      emitPlaybackState();
    } catch (e) {
      console.error('FM播放失败', e);
      playing.value = false;
      if (fmNextCallback) {
        fmNextCallback();
      } else {
        disableFmMode();
      }
    }
  }

  async function play(song: Song) {
    disableFmMode();

    const idx = queue.value.findIndex(s => s.id === song.id);
    if (idx !== -1 && idx === currentIndex.value && currentSong.value?.id === song.id) {
      if (!playing.value) {
        await invoke('resume_audio');
        playing.value = true;
        startTick();
      }
      return;
    }

    if (idx === -1) {
      queue.value.push(song);
      currentIndex.value = queue.value.length - 1;
    } else {
      currentIndex.value = idx;
    }
    await playCurrent();
  }

  async function playFromList(songs: Song[], startIndex: number) {
    disableFmMode();
    if (songs.length === 0) return;

    const targetSong = songs[startIndex];
    if (targetSong && currentSong.value?.id === targetSong.id && currentIndex.value >= 0) {
      const sameQueue = queue.value.length === songs.length
        && queue.value.every((s, i) => s.id === songs[i].id);
      if (sameQueue) {
        if (!playing.value) {
          await invoke('resume_audio');
          playing.value = true;
          startTick();
        }
        return;
      }
    }

    queue.value = [...songs];
    currentIndex.value = Math.max(0, Math.min(startIndex, songs.length - 1));
    await playCurrent();
  }

  let vipSkipCount = 0;
  const MAX_VIP_SKIP = 10;

  function waitForAudioStart(): Promise<void> {
    return new Promise<void>((resolve) => {
      _audioStartedResolve = resolve;
    });
  }

  async function playCurrent() {
    if (tickInterval) { clearInterval(tickInterval); setTickInterval(null); }
    reportScrobble();
    const song = queue.value[currentIndex.value];
    if (!song?.id) {
      console.error('无效的歌曲数据', song);
      return;
    }

    try {
      currentSong.value = song;
      playing.value = false;
      currentTime.value = 0;
      duration.value = (song.dt || 0) / 1000;

      if (song.localPath) {
        await invoke('play_local_audio', { path: song.localPath });
        await waitForAudioStart();
        playing.value = true;
      startTick();
      addRecent(song);
      emitPlaybackState();
      return;
    }

      const jsonStr: string = await invoke('get_song_url', { query: { id: Number(song.id), level: settings.audioQuality } });
      const data = JSON.parse(jsonStr);
      const url: string | undefined = data.url;

      if (!url) {
        console.error('未获取到有效播放地址', song);
        return;
      }

      if (data.freeTrialInfo) {
        console.warn('VIP 试听歌曲，自动跳过', song.name);
        showToast(`${song.name} 为 VIP 试听，已跳过`, 'info');
        vipSkipCount++;
        if (vipSkipCount >= MAX_VIP_SKIP) {
          console.warn('连续跳过 VIP 歌曲过多，停止跳过');
          vipSkipCount = 0;
          return;
        }
        next();
        return;
      }

      await invoke('play_audio', { url });
      await waitForAudioStart();
      playing.value = true;
      startTick();
      addRecent(song);
      vipSkipCount = 0;
      emitPlaybackState();
    } catch (e) {
      console.error('播放失败', e);
      playing.value = false;
    }
  }

  let onSeekStart: (() => void) | null = null;

  function startTick() {
    if (tickInterval) clearInterval(tickInterval);
    let seekGuard = false;
    onSeekStart = () => { seekGuard = true; };
    let syncCounter = 1;
    let lastSyncPos = -1;
    let backendFrozen = false;
    setTickInterval(setInterval(async () => {
      if (playing.value && duration.value > 0) {
        if (seekGuard) return;
        syncCounter++;
        if (syncCounter >= 2) {
          syncCounter = 0;
          try {
            const pos = await invoke<number>('get_audio_position');
            if (pos >= currentTime.value - 0.5) {
              currentTime.value = pos;
            }
            if (lastSyncPos < 0) {
              lastSyncPos = pos;
            } else if (pos <= lastSyncPos + 0.05) {
              backendFrozen = true;
              lastSyncPos = pos;
            } else {
              backendFrozen = false;
              lastSyncPos = pos;
            }
          } catch {}
        } else {
          if (!backendFrozen) {
            const next = currentTime.value + 0.25;
            if (next <= duration.value) {
              currentTime.value = next;
            }
          }
        }
        if (currentTime.value > duration.value) {
          currentTime.value = duration.value;
        }
      }
    }, 250));
  }

  async function toggle() {
    if (playing.value) {
      await invoke('pause_audio');
      playing.value = false;
    } else {
      await invoke('resume_audio');
      playing.value = true;
    }
    emitPlaybackState();
  }

  async function stop() {
    await invoke('stop_audio');
    playing.value = false;
    currentSong.value = null;
    currentTime.value = 0;
    if (tickInterval) { clearInterval(tickInterval); setTickInterval(null); }
    disableFmMode();
    emitPlaybackState();
  }


  function prev() {
    if (isFmMode.value) return;
    if (queue.value.length === 0) return;
    currentIndex.value = (currentIndex.value - 1 + queue.value.length) % queue.value.length;
    playCurrent();
  }

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
      stop();
      queue.value.splice(index, 1);
      if (queue.value.length === 0) {
        currentIndex.value = -1;
        return;
      }
      if (currentIndex.value >= queue.value.length) {
        currentIndex.value = queue.value.length - 1;
      }
    } else {
      queue.value.splice(index, 1);
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
      currentTime.value = time;
      if (onSeekStart) onSeekStart();
      await invoke('seek_audio', { time });
      startTick();
      emitPlaybackState();
    } catch (e) {
      console.error('seek 失败', e);
    }
  }

  async function adjustVolume(delta: number) {
    const newVol = Math.max(0, Math.min(100, volume.value + delta));
    volume.value = newVol;
    await invoke('set_volume', { vol: newVol / 100 });
    emitPlaybackState();
  }


  const playMode = ref<PlayMode>('loop');

  function setPlayMode(mode: PlayMode) {
    playMode.value = mode;
  }

  function next() {
    if (isFmMode.value && fmNextCallback) {
      fmNextCallback();
      return;
    }

    if (queue.value.length === 0) return;
    let nextIndex: number;
    switch (playMode.value) {
      case 'repeat-one':
        playCurrent();
        return;
      case 'shuffle':
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
        nextIndex = (currentIndex.value + 1) % queue.value.length;
        break;
    }
    currentIndex.value = nextIndex;
    playCurrent();
  }

  const showRoamDrawer = ref(false);
  const roamInitialTab = ref<'lyric' | 'comment'>('lyric');
  const commentSongId = ref<number | null>(null);

  function openRoamDrawer(tab: 'lyric' | 'comment' = 'lyric') {
    roamInitialTab.value = tab;
    showRoamDrawer.value = true;
    nextTick(() => { roamInitialTab.value = 'lyric'; });
  }

  function openCommentForSong(songId: number) {
    commentSongId.value = songId;
    openRoamDrawer('comment');
  }

  function closeRoamDrawer() {
    showRoamDrawer.value = false;
  }

  function toggleRoamDrawer() {
    showRoamDrawer.value = !showRoamDrawer.value;
  }

  async function loadFirstFmSong() {
  try {
    const batch = await fetchFmBatch();
    if (batch.length > 0) {
      fmQueue.push(...batch);
      const song = fmQueue.shift()!;
      enableFmMode(nextFm);
      await playFmSong(song);
      return true;
    }
  } catch (e) {
    console.error(e);
  }
  return false;
}


// -------- FM 专属状态 --------
const fmSong = ref<Song | null>(null);
const fmPlaying = ref(false);

async function loadFm() {
  try {
    if (fmQueue.length === 0) {
      const batch = await fetchFmBatch();
      if (batch.length === 0) return;
      fmQueue.push(...batch);
    }
    const song = fmQueue.shift()!;
    fmSong.value = song;
    enableFmMode(nextFm);
    await playFmSong(song);
    fmPlaying.value = true;
    if (fmQueue.length <= 1) {
      fetchFmBatch().then(batch => { fmQueue.push(...batch); }).catch(() => {});
    }
  } catch (e) {
    console.error('FM加载失败', e);
  }
}

async function toggleFm() {
  if (!fmSong.value) return;
  if (fmPlaying.value) {
    await toggle();
    fmPlaying.value = playing.value;
  } else {
    if (currentSong.value?.id === fmSong.value.id) {
      await toggle();
      fmPlaying.value = playing.value;
    } else {
      enableFmMode(nextFm);
      await playFmSong(fmSong.value);
      fmPlaying.value = true;
    }
  }
}

async function nextFm() {
  await loadFm();
}

let _audioStartedResolve: (() => void) | null = null;
let _tickInterval: ReturnType<typeof setInterval> | null = null;

listen('audio-started', () => {
  if (_audioStartedResolve) {
    _audioStartedResolve();
    _audioStartedResolve = null;
  }
});

listen('audio-ended', () => {
  if (_tickInterval) { clearInterval(_tickInterval); _tickInterval = null; }
  const player = usePlayerStore();
  player.reportScrobble();
  player.next();
});

listen<string>('mpris-command', (event) => {
  const cmd = event.payload;
  const player = usePlayerStore();
  if (cmd === 'Next') {
    player.next();
  } else if (cmd === 'Previous') {
    player.prev();
  } else if (cmd === 'PlayPause') {
    player.toggle();
  } else if (cmd === 'Play') {
    if (!player.playing) player.toggle();
  } else if (cmd === 'Pause') {
    if (player.playing) player.toggle();
  } else if (cmd === 'Stop') {
    player.stop();
  } else if (cmd.startsWith('SetVolume:')) {
    const vol = parseFloat(cmd.slice(10));
    if (!isNaN(vol)) {
      player.volume = Math.round(vol * 100);
      invoke('set_volume', { vol }).catch(() => {});
    }
  } else if (cmd.startsWith('Seek:')) {
    const offsetUs = parseInt(cmd.slice(5), 10);
    const offsetSec = offsetUs / 1_000_000;
    const newPos = Math.max(0, Math.min(player.currentTime + offsetSec, player.duration));
    player.seek(newPos);
  } else if (cmd.startsWith('SetPosition:')) {
    const posUs = parseInt(cmd.slice(13), 10);
    const posSec = posUs / 1_000_000;
    if (posSec < 1 && player.currentTime > 5) {
      return;
    }
    player.seek(posSec);
  } else if (cmd === 'Raise') {
    getCurrentWindow().show().catch(() => {});
    getCurrentWindow().setFocus().catch(() => {});
  } else if (cmd === 'Quit') {
    getCurrentWindow().close().catch(() => {});
  }
});

watch(currentSong, (newSong) => {
  if (isFmMode.value && newSong?.id !== fmSong.value?.id) {
    fmPlaying.value = false;
    disableFmMode();
  }
});

watch(playing, (val) => {
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
    playFromList,
    playAll,
    toggle,
    stop,
    prev,
    next,
    seek,
    playCurrent,
    volume,
    adjustVolume,

    removeFromQueue,
    clearQueue,

    recentLocal,

    likedIds,
    isLiked,
    loadLikedIds,
    toggleLike,

    showRoamDrawer,
    roamInitialTab,
    commentSongId,
    openCommentForSong,
    openRoamDrawer,
    closeRoamDrawer,
    toggleRoamDrawer,
    loadFirstFmSong,

    reportScrobble,

    fmSong,
    fmPlaying,
    loadFm,
    toggleFm,
    nextFm,
  };
});
