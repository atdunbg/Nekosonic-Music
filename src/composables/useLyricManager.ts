import { ref } from 'vue';
import { SongApi } from '../api/song';
import { parseLrc, mergeTranslation, getCurrentLyricIndex, type LyricLine } from '../utils/lyric';
import type { Song } from '../types/song';

/** 已解析的歌词数据 */
export interface LyricData {
  lines: LyricLine[];
  hasTranslation: boolean;
}

const MAX_CACHE = 100;
/** 歌词持久化 TTL：7 天（歌词很少变动） */
const LYRIC_TTL = 7 * 24 * 60 * 60 * 1000;
const LYRIC_LS_PREFIX = 'lyric:';

/** L1 内存缓存（按 songId 索引） */
const lyricCache = new Map<number, LyricData>();

/** 从 L2（localStorage）读取歌词并校验 TTL */
function lyricLsGet(songId: number): LyricData | null {
  try {
    const raw = localStorage.getItem(LYRIC_LS_PREFIX + songId);
    if (!raw) return null;
    const entry = JSON.parse(raw);
    if (Date.now() - entry.ts > LYRIC_TTL) {
      localStorage.removeItem(LYRIC_LS_PREFIX + songId);
      return null;
    }
    return entry.data as LyricData;
  } catch {
    return null;
  }
}

/** 写入 L2（localStorage），配额不足时静默失败 */
function lyricLsSet(songId: number, data: LyricData) {
  try {
    localStorage.setItem(LYRIC_LS_PREFIX + songId, JSON.stringify({ data, ts: Date.now() }));
  } catch { /* 配额不足：仅保留内存缓存 */ }
}

/** 当前歌词请求序列号（防竞态） */
let lyricReqSeq = 0;

/** 正在加载中的请求（去重） */
const pendingRequests = new Map<number, Promise<LyricData>>();

/**
 * 解析原始歌词 JSON 为 LyricData
 */
function parseLyricJson(jsonStr: string): LyricData {
  const data = JSON.parse(jsonStr);
  const lrc = data?.lrc?.lyric || '';
  const tLrc = data?.tlyric?.lyric || '';
  let parsed = lrc ? parseLrc(lrc) : [];
  let hasTranslation = false;
  if (tLrc && parsed.length > 0) {
    parsed = mergeTranslation(parsed, tLrc);
    hasTranslation = parsed.some(l => l.translation);
  }
  return { lines: parsed, hasTranslation };
}

/**
 * 获取歌词数据（带缓存 + 请求去重）
 *
 * @param songId 歌曲 ID
 * @param force 是否强制刷新缓存
 */
async function fetchLyric(songId: number, force = false): Promise<LyricData> {
  // L1 内存
  if (!force && lyricCache.has(songId)) {
    return lyricCache.get(songId)!;
  }
  // L2 localStorage（回填 L1）
  if (!force) {
    const l2 = lyricLsGet(songId);
    if (l2) {
      if (lyricCache.size >= MAX_CACHE) {
        const firstKey = lyricCache.keys().next().value;
        if (firstKey !== undefined) lyricCache.delete(firstKey);
      }
      lyricCache.set(songId, l2);
      return l2;
    }
  }
  if (pendingRequests.has(songId)) {
    return pendingRequests.get(songId)!;
  }

  const promise = (async () => {
    const jsonStr = await SongApi.getLyric(songId);
    const data = parseLyricJson(jsonStr);
    // 写入 L1 + L2（LRU 策略：超限时删除最早条目）
    if (lyricCache.size >= MAX_CACHE) {
      const firstKey = lyricCache.keys().next().value;
      if (firstKey !== undefined) lyricCache.delete(firstKey);
    }
    lyricCache.set(songId, data);
    lyricLsSet(songId, data);
    return data;
  })().finally(() => {
    pendingRequests.delete(songId);
  });

  pendingRequests.set(songId, promise);
  return promise;
}

/**
 * 歌词管理器
 *
 * 职责：
 * - 跟随 currentSong 自动加载歌词（带防竞态）
 * - 跟随 currentTime 自动计算当前行
 * - 内存缓存（避免重复请求）
 * - 预加载下一首歌词
 */
export function useLyricManager() {
  const lyrics = ref<LyricLine[]>([]);
  const currentLyricIdx = ref(-1);
  const showTranslation = ref(true);
  const hasTranslation = ref(false);

  /** 当前加载的 songId（用于防竞态） */
  let currentSongId: number | null = null;

  /** 加载指定歌曲的歌词 */
  async function loadLyric(song: Song | null) {
    if (!song) {
      lyrics.value = [];
      currentLyricIdx.value = -1;
      hasTranslation.value = false;
      currentSongId = null;
      return;
    }

    const seq = ++lyricReqSeq;
    currentSongId = song.id;

    try {
      const data = await fetchLyric(song.id);
      // 防竞态：如果期间又切了歌，丢弃本次结果
      if (seq !== lyricReqSeq || currentSongId !== song.id) return;
      lyrics.value = data.lines;
      hasTranslation.value = data.hasTranslation;
      currentLyricIdx.value = -1;
    } catch {
      if (seq !== lyricReqSeq) return;
      lyrics.value = [];
      hasTranslation.value = false;
    }
  }

  /** 预加载指定歌曲的歌词（不更新当前显示） */
  function preloadLyric(song: Song | null) {
    if (!song) return;
    fetchLyric(song.id).catch(() => { /* 预加载失败静默 */ });
  }

  /** 根据当前播放时间更新当前歌词行 */
  function updateCurrentIndex(time: number) {
    if (lyrics.value.length === 0) return;
    const idx = getCurrentLyricIndex(lyrics.value, time);
    if (idx !== currentLyricIdx.value) {
      currentLyricIdx.value = idx;
    }
  }

  function toggleTranslation() {
    showTranslation.value = !showTranslation.value;
  }

  return {
    lyrics,
    currentLyricIdx,
    showTranslation,
    hasTranslation,
    loadLyric,
    preloadLyric,
    updateCurrentIndex,
    toggleTranslation,
  };
}
