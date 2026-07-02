import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { normalizeSong } from '../utils/song';
import type { Song, PlayMode } from '../types/song';
import { useSettingsStore } from './settings';
import { useLikedStore } from './liked';
import { useRecentStore } from './recent';
import { showToast } from '../composables/useToast';
import { MusicApi, AudioApi } from '../api';

import { listen, emit } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const usePlayerStore = defineStore('player', () => {
  const currentSong = ref<Song | null>(null);
  const playing = ref(false);
  const currentTime = ref(0);
  const duration = ref(0);

  const queue = ref<Song[]>([]);
  const currentIndex = ref(-1);

  /** 当前播放来源（用于卡片显示播放/暂停状态） */
  const currentSource = ref<{ type: 'playlist' | 'album' | 'songs' | 'fm' | null; id: number | null }>({
    type: null,
    id: null,
  });

  const settings = useSettingsStore();
  const volume = ref(settings.volume);

  watch(volume, (val) => { settings.volume = val; });

  const recentStore = useRecentStore();
  const likedStore = useLikedStore();
  const { addRecent } = recentStore;

  let tickInterval: ReturnType<typeof setInterval> | null = null;
  let tickGeneration = 0;
  function clearTick() {
    if (tickInterval) { clearInterval(tickInterval); tickInterval = null; }
    tickGeneration++;
  }
  function setTick(v: ReturnType<typeof setInterval>) {
    tickInterval = v;
  }

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

  const isFmMode = ref(false);
  const fmQueue: Song[] = [];
  let fmNextCallback: (() => void) | null = null;

  const fmMode = ref<string>('DEFAULT');
  const fmSubMode = ref<string>('');

  let lastScrobbleId: number | null = null;
  let lastScrobbleStartTime: number = 0;
  let lastScrobbleAlg: string | undefined;
  let lastScrobbleSource: string | undefined;
  let lastScrobbleBitrate: number | undefined;

  /// 上报上一首歌的听歌记录（scrobble），然后记录当前歌的开始时间
  function reportScrobble() {
    // 先上报：如果有正在记录的歌曲且播放超过 5 秒，发送 scrobble
    if (lastScrobbleId != null && lastScrobbleStartTime > 0) {
      const playedSec = Math.round((Date.now() - lastScrobbleStartTime) / 1000);
      if (playedSec > 5 && navigator.onLine) {
        MusicApi.scrobble({
          id: lastScrobbleId,
          sourceid: isFmMode.value ? String(lastScrobbleId) : '',
          time: playedSec,
          alg: lastScrobbleAlg || '',
          source: lastScrobbleSource || 'list',
          bitrate: lastScrobbleBitrate || 0,
        }).catch(() => {});
      }
    }
    // 再记录：当前歌曲作为新的 scrobble 起点
    const song = currentSong.value;
    if (!song || song.localPath || song.id == null) {
      lastScrobbleId = null;
      lastScrobbleStartTime = 0;
      lastScrobbleAlg = undefined;
      lastScrobbleSource = undefined;
      lastScrobbleBitrate = undefined;
    } else {
      lastScrobbleId = song.id;
      lastScrobbleStartTime = Date.now();
      lastScrobbleAlg = song.alg;
      lastScrobbleSource = isFmMode.value ? 'personal_fm' : 'list';
      lastScrobbleBitrate = song.br;
    }
  }

  function enableFmMode(onNext: () => void) {
    isFmMode.value = true;
    fmNextCallback = onNext;
  }

  function disableFmMode() {
    isFmMode.value = false;
    fmNextCallback = null;
    fmQueue.length = 0;
    fmMode.value = 'DEFAULT';
    fmSubMode.value = '';
    fmSong.value = null;
    fmPlaying.value = false;
  }

  function clearFmQueue() {
    fmQueue.length = 0;
  }

  async function fmTrash(songId: number) {
    try {
      await MusicApi.fmTrash(songId, 25);
    } catch (e) {
      console.error('fm_trash 失败', e);
      showToast('减少推荐失败', 'error');
    }
    await nextFm();
  }

  async function fetchFmBatch(): Promise<Song[]> {
    const isDefault = fmMode.value === 'DEFAULT' && !fmSubMode.value;
    const jsonStr: string = isDefault
      ? await MusicApi.personalFm()
      : await MusicApi.personalFmMode({
          mode: fmMode.value,
          subMode: fmSubMode.value,
          limit: 3,
        });
    const data = JSON.parse(jsonStr);
    const raw = data.data || data;
    if (!Array.isArray(raw) || raw.length === 0) return [];
    return raw.map((s: any) => normalizeSong(s));
  }

  let fmVipSkipCount = 0;
  const MAX_FM_VIP_SKIP = 10;

  async function playFmSong(song: Song) {
    const seq = ++_playSeq;
    switching.value = true;
    clearTick();
    reportScrobble();
    if (!song.dt || song.dt === 0) {
      try {
        const jsonStr = await MusicApi.getSongDetail(String(song.id));
        const data = JSON.parse(jsonStr);
        const full = data.songs?.[0];
        if (full) {
          song.dt = full.dt || 0;
          song.al = full.al || song.al;
          song.ar = full.ar || song.ar;
        }
      } catch { /* 忽略 */ }
    }

    if (!(await stopAndWaitStopped(seq))) return;
    queue.value = [];
    currentIndex.value = -1;
    playing.value = false;

    fmSong.value = song;
    currentSong.value = song;
    try {
      const data = await loadSongUrl(song, true);
      if (seq !== _playSeq) return;
      const url: string | undefined = data.url;
      // 记录实际播放音源（netease / 波点音乐 / 酷狗音乐 / ...）
      if (data.source) song.source = data.source;
      if (!url) {
        const fee = data.fee;
        if (fee === 4) {
          showToast(`${song.name} 为数字专辑，已跳过`, 'info');
        } else if (fee === 1) {
          showToast(`${song.name} 为 VIP 专属歌曲，已跳过`, 'info');
        } else {
          showToast(`${song.name} 暂无播放源`, 'info');
        }
        fmVipSkipCount++;
        if (fmVipSkipCount >= MAX_FM_VIP_SKIP) {
          fmVipSkipCount = 0;
          disableFmMode();
          return;
        }
        switching.value = false;
        if (fmNextCallback) {
          fmNextCallback();
        } else {
          disableFmMode();
        }
        return;
      }

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
        switching.value = false;
        if (fmNextCallback) {
          fmNextCallback();
        } else {
          disableFmMode();
        }
        return;
      }

      fmVipSkipCount = 0;
      await AudioApi.playAudio(url);
      if (seq !== _playSeq) return;
      const started = await waitForPlaybackStart();
      if (seq !== _playSeq) return;
      if (started) {
        onPlaybackStarted(song, true);
      } else {
        playing.value = false;
        showToast('FM 播放启动超时，仍在尝试加载…', 'info');
        watchForLatePlayback(seq, song);
      }
    } catch (e) {
      if (seq !== _playSeq) return;
      console.error('FM播放失败', e);
      playing.value = false;
      showToast('FM 播放失败', 'error');
      if (fmNextCallback) {
        fmNextCallback();
      } else {
        disableFmMode();
      }
    } finally {
      if (seq === _playSeq) {
        switching.value = false;
      }
    }
  }

  async function play(song: Song) {
    disableFmMode();

    const idx = queue.value.findIndex(s => s.id === song.id);
    if (idx !== -1 && idx === currentIndex.value && currentSong.value?.id === song.id) {
      if (!playing.value) {
        await AudioApi.resumeAudio();
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

  async function playFromList(songs: Song[], startIndex: number, source?: { type: 'playlist' | 'album' | 'songs'; id?: number }) {
    disableFmMode();
    if (songs.length === 0) return;

    const targetSong = songs[startIndex];
    if (targetSong && currentSong.value?.id === targetSong.id && currentIndex.value >= 0) {
      const sameQueue = queue.value.length === songs.length
        && queue.value.every((s, i) => s.id === songs[i].id);
      if (sameQueue) {
        // 同一首歌且同队列：切换播放/暂停
        if (playing.value) {
          await AudioApi.pauseAudio();
          playing.value = false;
          clearTick();
        } else {
          await AudioApi.resumeAudio();
          playing.value = true;
          startTick();
        }
        return;
      }
    }

    queue.value = [...songs];
    currentIndex.value = Math.max(0, Math.min(startIndex, songs.length - 1));
    currentSource.value = { type: source?.type ?? null, id: source?.id ?? null };
    await playCurrent();
  }

  let vipSkipCount = 0;
  const MAX_VIP_SKIP = 10;

  let _playSeq = 0;
  const switching = ref(false);

  /// 停止后端播放并等待 is_playing 变为 false（最多 1.5s）。
  /// 替代音源的 loadSongUrl（跨源 fallback）耗时可能数秒，
  /// 若不先停止旧歌，期间后端 is_playing 仍为 true、shared_position 持续增长，
  /// 之后 waitForPlaybackStart 会误判旧歌的 is_playing=true 为新歌启动，
  /// 导致 startTick 后 tick 读取旧歌的 position 覆盖 currentTime（进度条不从零开始）。
  /// 返回 false 表示 seq 已过期，调用方应立即 return。
  async function stopAndWaitStopped(seq: number): Promise<boolean> {
    try { await AudioApi.stopAudio(); } catch { /* 忽略 */ }
    const deadline = Date.now() + 1500;
    while (Date.now() < deadline) {
      if (seq !== _playSeq) return false;
      try {
        if (!(await AudioApi.isAudioPlaying())) return true;
      } catch { return true; }
      await new Promise(r => setTimeout(r, 50));
    }
    return true;
  }

  /// 等待新播放启动（is_playing 变为 true）。
  /// 调用方须先用 stopAndWaitStopped 确保旧播放已停止，否则可能命中旧歌的 true。
  async function waitForPlaybackStart(timeoutMs: number = 8000): Promise<boolean> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      await new Promise(r => setTimeout(r, 100));
      try {
        if (await AudioApi.isAudioPlaying()) return true;
      } catch { /* 忽略 */ }
    }
    try {
      return await AudioApi.isAudioPlaying();
    } catch { return false; }
  }

  /// 播放启动成功后的统一处理：设置状态、启动 tick、记录最近播放、上报状态
  function onPlaybackStarted(song: Song, isFm: boolean) {
    playing.value = true;
    if (isFm) duration.value = (song.dt || 0) / 1000;
    currentTime.value = 0;
    startTick();
    addRecent(song);
    vipSkipCount = 0;
    emitPlaybackState();
  }

  /**
   * 统一获取播放 URL：外部音源歌曲走 getExternalSongUrl，网易云走 getSongUrl（含 fallback）
   * 返回结构兼容网易云 data 对象（url/source/fee/freeTrialInfo）
   */
  async function loadSongUrl(song: Song, fmMode = false): Promise<{ url?: string; source?: string; fee?: number; freeTrialInfo?: any }> {
    // 外部音源歌曲：直接通过对应源取 URL，失败时后端会跨源 fallback
    if (song.externalSourceId && song.externalId) {
      const sourceCfg = settings.musicSources.find(s => s.id === song.externalSourceId);
      if (!sourceCfg) {
        console.warn('外部音源配置未找到:', song.externalSourceId);
        return { url: undefined };
      }
      try {
        const json = await MusicApi.getExternalSongUrl({
          source: sourceCfg,
          songId: song.externalId,
          quality: settings.audioQuality,
          songName: song.name,
          artist: song.ar?.map(a => a.name).join(' / ') || '',
          allSources: settings.enabledMusicSources,
        });
        const data = JSON.parse(json);
        return {
          url: data.url,
          source: data.source_label || sourceCfg.label,
        };
      } catch (e) {
        console.error('外部音源获取URL失败:', e);
        return { url: undefined };
      }
    }
    // 网易云流程（含 fallback）
    const jsonStr = await MusicApi.getSongUrl({
      id: Number(song.id),
      level: settings.audioQuality,
      fm_mode: fmMode,
      sources: settings.enabledMusicSources,
    });
    const data = JSON.parse(jsonStr);
    return {
      url: data.url,
      source: data.source,
      fee: data.fee,
      freeTrialInfo: data.freeTrialInfo,
    };
  }

  async function playCurrent() {
    const seq = ++_playSeq;
    switching.value = true;
    clearTick();
    reportScrobble();

    if (!(await stopAndWaitStopped(seq))) return;

    const song = queue.value[currentIndex.value];
    if (!song?.id) {
      console.error('无效的歌曲数据', song);
      switching.value = false;
      return;
    }

    try {
      currentSong.value = song;
      playing.value = false;
      currentTime.value = 0;
      duration.value = (song.dt || 0) / 1000;

      if (song.localPath) {
        await AudioApi.playLocalAudio(song.localPath);
        if (seq !== _playSeq) return;
        const started = await waitForPlaybackStart();
        if (seq !== _playSeq) return;
        if (started) {
          onPlaybackStarted(song, false);
        } else {
          showToast('播放启动超时，仍在尝试加载…', 'info');
          watchForLatePlayback(seq, song);
        }
        return;
      }

      const data = await loadSongUrl(song, false);
      if (seq !== _playSeq) return;
      const url: string | undefined = data.url;
      // 记录实际播放音源（netease / 波点音乐 / 酷狗音乐 / ...）
      if (data.source) song.source = data.source;

      if (!url) {
        // url 为空：可能是数字专辑/付费歌曲，根据 fee 字段判断
        const fee = data.fee;
        if (fee === 4) {
          showToast(`${song.name} 为数字专辑，需购买后播放`, 'info');
        } else if (fee === 1) {
          showToast(`${song.name} 为 VIP 专属歌曲`, 'info');
        } else {
          showToast(`${song.name} 暂无播放源`, 'info');
        }
        vipSkipCount++;
        if (vipSkipCount >= MAX_VIP_SKIP) {
          vipSkipCount = 0;
          return;
        }
        switching.value = false;
        next();
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
        switching.value = false;
        next();
        return;
      }

      await AudioApi.playAudio(url);
      if (seq !== _playSeq) return;
      const started = await waitForPlaybackStart();
      if (seq !== _playSeq) return;
      if (started) {
        onPlaybackStarted(song, false);
      } else {
        playing.value = false;
        showToast('播放启动超时，仍在尝试加载…', 'info');
        watchForLatePlayback(seq, song);
      }
    } catch (e) {
      if (seq !== _playSeq) return;
      console.error('播放失败', e);
      playing.value = false;
      showToast('播放失败，请稍后重试', 'error');
    } finally {
      if (seq === _playSeq) {
        switching.value = false;
      }
    }
  }

  /// 超时后继续监听后端播放状态，如果后端实际开始播放则恢复状态
  function watchForLatePlayback(seq: number, song: Song) {
    let attempts = 0;
    const maxAttempts = 15;
    const check = async () => {
      if (seq !== _playSeq) return;
      if (playing.value) return;
      attempts++;
      if (attempts > maxAttempts) return;
      try {
        const backendPlaying = await AudioApi.isAudioPlaying();
        if (seq !== _playSeq) return;
        if (backendPlaying) {
          playing.value = true;
          startTick();
          addRecent(song);
          vipSkipCount = 0;
          emitPlaybackState();
          return;
        }
      } catch { /* 忽略 */ }
      if (seq === _playSeq && !playing.value) {
        setTimeout(check, 1000);
      }
    };
    setTimeout(check, 1000);
  }

  let onSeekStart: (() => void) | null = null;

  function startTick() {
    clearTick();
    const gen = tickGeneration;
    let seekGuard = false;
    onSeekStart = () => { seekGuard = true; };
    let syncCounter = 1;
    let lastSyncPos = -1;
    let backendFrozen = false;
    let stateSyncCounter = 0;
    setTick(setInterval(async () => {
      if (gen !== tickGeneration) return;
      if (playing.value && duration.value > 0) {
        if (seekGuard) return;
        syncCounter++;
        stateSyncCounter++;
        if (syncCounter >= 2) {
          syncCounter = 0;
          try {
            const pos = await AudioApi.getAudioPosition();
            if (gen !== tickGeneration) return;
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
          } catch { /* 忽略 */ }

          if (stateSyncCounter >= 4) {
            stateSyncCounter = 0;
            try {
              const backendPlaying = await AudioApi.isAudioPlaying();
              if (gen !== tickGeneration) return;
              if (backendPlaying !== playing.value) {
                playing.value = backendPlaying;
              }
            } catch { /* 忽略 */ }
          }
        } else {
          if (!backendFrozen) {
            const next = currentTime.value + 0.5;
            if (next <= duration.value) {
              currentTime.value = next;
            }
          }
        }
        if (currentTime.value > duration.value) {
          currentTime.value = duration.value;
        }
      }
    }, 500));
  }

  async function toggle() {
    if (switching.value) return;
    try {
      const backendPlaying = await AudioApi.isAudioPlaying();
      if (backendPlaying !== playing.value) {
        playing.value = backendPlaying;
      }
    } catch { /* 忽略查询失败 */ }

    if (playing.value) {
      await AudioApi.pauseAudio();
      playing.value = false;
    } else {
      await AudioApi.resumeAudio();
      playing.value = true;
    }
    emitPlaybackState();
  }

  async function stop() {
    await AudioApi.stopAudio();
    playing.value = false;
    currentSong.value = null;
    currentTime.value = 0;
    clearTick();
    disableFmMode();
    emitPlaybackState();
  }


  function prev() {
    if (isFmMode.value) return;
    if (switching.value) return;
    if (queue.value.length === 0) return;
    currentIndex.value = (currentIndex.value - 1 + queue.value.length) % queue.value.length;
    playCurrent();
  }

  async function playAll(songs: Song[], source?: { type: 'playlist' | 'album' | 'songs'; id?: number }) {
    if (songs.length === 0) return;
    queue.value = [...songs];
    currentIndex.value = 0;
    currentSource.value = { type: source?.type ?? null, id: source?.id ?? null };
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
    if (switching.value) return;
    try {
      currentTime.value = time;
      if (onSeekStart) onSeekStart();
      await AudioApi.seekAudio(time);
      startTick();
      emitPlaybackState();
    } catch (e) {
      console.error('seek 失败', e);
    }
  }

  async function adjustVolume(delta: number) {
    const newVol = Math.max(0, Math.min(100, volume.value + delta));
    volume.value = newVol;
    await AudioApi.setVolume(newVol / 100);
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

    if (switching.value) return;
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
    console.error('FM 加载下一首失败', e);
    showToast('FM 加载失败', 'error');
  }
  return false;
}

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
    showToast('FM 加载失败', 'error');
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

listen('audio-ended', () => {
  if (switching.value) return;
  const player = usePlayerStore();
  player.clearTick();
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
      AudioApi.setVolume(vol).catch(() => {});
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
    switching,
    currentTime,
    duration,
    queue,
    currentIndex,
    currentSource,
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

    // 喜欢列表（委托给 liked store）
    get likedIds() { return likedStore.likedIds; },
    isLiked: likedStore.isLiked,
    loadLikedIds: likedStore.loadLikedIds,
    toggleLike: likedStore.toggleLike,

    // 最近播放（委托给 recent store）
    get recentLocal() { return recentStore.recentLocal; },
    addRecent,

    loadFirstFmSong,

    fetchFmBatch,
    clearFmQueue,
    fmTrash,
    reportScrobble,
    clearTick,

    fmSong,
    fmPlaying,
    fmMode,
    fmSubMode,
    loadFm,
    toggleFm,
    nextFm,
  };
});
