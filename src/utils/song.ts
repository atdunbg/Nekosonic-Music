export type { Song } from '../types/song';
import type { Song } from '../types/song';

export function normalizeSong(song: any): Song {
  const al = {
    id: song.al?.id || song.album?.id,
    picUrl: song.al?.picUrl || song.album?.picUrl || '',
    name: song.al?.name || song.album?.name,
  };
  const rawAr = (song.ar && song.ar.length > 0) ? song.ar : (song.artists || []);
  // 过滤掉 id 和 name 同时不存在的歌手（下线艺人等）
  const ar = rawAr.filter((a: any) => a.name);
  let dt = song.dt || song.duration || 0;
  if (dt < 100 || dt > 7200000) dt = 0;
  // fee / maxbr 可能直接在 song 上，也可能在 song.privilege 里
  // （/cloudsearch 附在 song 上，/artist/songs / /playlist/detail 等可能在单独 privileges 数组）
  const priv = song.privilege || {};
  return {
    id: song.id,
    name: song.name,
    ar,
    al,
    dt,
    localPath: song.localPath,
    alg: song.alg || undefined,
    br: song.br || undefined,
    fee: typeof song.fee === 'number' ? song.fee : (typeof priv.fee === 'number' ? priv.fee : undefined),
    maxbr: priv.maxbr || song.maxbr || undefined,
  };
}

/**
 * 将网易云单独的 privileges 数组合并到 songs 中（给每个 song 附上对应 privilege）
 * 用于 /artist/songs / /playlist/detail 等接口，privilege 不在 song 上而是单独数组
 */
export function attachPrivileges(songs: any[], privileges?: any[]): any[] {
  if (!Array.isArray(privileges) || !privileges.length) return songs;
  const map = new Map<number, any>();
  for (const p of privileges) {
    if (p?.id != null) map.set(p.id, p);
  }
  return songs.map(s => (s && !s.privilege && s.id != null && map.has(s.id))
    ? { ...s, privilege: map.get(s.id) }
    : s);
}

/** 将网易云单独的 privileges 数组合并到 songs 并 normalize（5 个详情页共用） */
export function normalizeSongsWithPrivileges(songs: any[], privileges?: any[]): Song[] {
  return attachPrivileges(songs, privileges).map(normalizeSong);
}

/** 歌曲标签类型 */
export type SongTagKind = 'vip' | 'album' | 'hires' | 'sq' | 'hq' | 'original' | 'cover' | 'live' | 'accompaniment' | 'dj';

export interface SongTag {
  /** 标签文案 */
  label: string;
  /** 标签种类，便于按需样式覆盖 */
  kind: SongTagKind;
  /** Tailwind class 字符串（背景 + 文字色） */
  class: string;
}

// getSongTags 版本标签正则（模块级常量，避免每次调用重建）
// 注意：JS 正则的 \b 只对 ASCII 字符生效，所以中文字符周围不能用 \b
const RE_COVER = /翻唱|\bcover\b/i;
const RE_ORIGINAL = /原唱/;
const RE_LIVE = /\blive\b|现场|演唱会|巡演|concert/i;
const ACCOMPANIMENT_RE = /伴奏|karaoke|\binst\b/i;
const REMIX_RE = /\b(?:dj|remix)\b|混音|电音|dj版|混音版/i;

/**
 * 计算歌曲应显示的标签
 * 顺序：付费 → 音质 → 版本（原唱/翻唱/Live/伴奏/DJ）
 * 参考 SPlayer 的标签展示
 */
export function getSongTags(song: Song): SongTag[] {
  const tags: SongTag[] = [];
  const name = song.name || '';
  const lower = name.toLowerCase();

  // 1) 付费标签（网易云 fee 字段）
  // 注意：fee=8 实际表示「免费低音质版」，大量免费歌曲都是 8，不能当试听
  //   - fee=1: VIP 歌曲
  //   - fee=4: 数字专辑
  //   - fee=0/8: 免费歌曲，不显示付费标签
  if (song.fee === 1) {
    tags.push({ label: 'VIP', kind: 'vip', class: 'bg-danger/15 text-danger' });
  } else if (song.fee === 4) {
    tags.push({ label: '专辑', kind: 'album', class: 'bg-warning/15 text-warning' });
  }

  // 2) 音质标签：优先用 maxbr（最大可达码率），fallback 到 br（实际码率）
  const qualityBr = song.maxbr || song.br || 0;
  if (qualityBr >= 1999000) {
    tags.push({ label: 'Hi-Res', kind: 'hires', class: 'bg-info/15 text-info' });
  } else if (qualityBr >= 999000) {
    tags.push({ label: 'SQ', kind: 'sq', class: 'bg-warning/15 text-warning' });
  } else if (qualityBr >= 320000) {
    tags.push({ label: 'HQ', kind: 'hq', class: 'bg-accent/15 text-accent-text' });
  }

  // 3) 版本标签（从歌名推断，互斥：原唱/翻唱 二选一）
  const isCover = RE_COVER.test(lower);
  const isOriginal = RE_ORIGINAL.test(name);
  const isLive = RE_LIVE.test(lower);
  const isAccompaniment = ACCOMPANIMENT_RE.test(lower);
  const isDj = REMIX_RE.test(lower);

  if (isCover) {
    tags.push({ label: '翻唱', kind: 'cover', class: 'bg-info/15 text-info' });
  } else if (isOriginal) {
    tags.push({ label: '原唱', kind: 'original', class: 'bg-accent/15 text-accent-text' });
  }
  if (isLive) {
    tags.push({ label: 'Live', kind: 'live', class: 'bg-accent/15 text-accent-text' });
  }
  if (isAccompaniment) {
    tags.push({ label: '伴奏', kind: 'accompaniment', class: 'bg-muted text-content-2' });
  }
  if (isDj) {
    tags.push({ label: 'DJ', kind: 'dj', class: 'bg-warning/15 text-warning' });
  }

  return tags;
}

export function getCoverUrl(song: Song | null, sizeParam = ''): string {
  if (!song) return '';
  const raw = song.al?.picUrl || '';
  if (!raw) return '';
  if (!sizeParam || raw.startsWith('data:')) return raw;
  return raw + sizeParam;
}

export function getArtistDisplay(song: Song): string {
  if (!song.ar || song.ar.length === 0) return '未知歌手';
  const names = song.ar
    .filter(a => a.id != null && a.name)
    .map(a => a.name);
  return names.length > 0 ? names.join(' / ') : '未知歌手';
}

const colorCache = new Map<string, string>();
const MAX_COLOR_CACHE = 200;

export function extractDominantColor(imageUrl: string): Promise<string> {
  if (colorCache.has(imageUrl)) {
    return Promise.resolve(colorCache.get(imageUrl)!);
  }

  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const size = 8;
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext('2d');
        if (!ctx) { resolve(''); return; }
        ctx.drawImage(img, 0, 0, size, size);
        const data = ctx.getImageData(0, 0, size, size).data;

        let r = 0, g = 0, b = 0, count = 0;
        for (let i = 0; i < data.length; i += 4) {
          r += data[i];
          g += data[i + 1];
          b += data[i + 2];
          count++;
        }
        r = Math.round(r / count);
        g = Math.round(g / count);
        b = Math.round(b / count);

        const color = `rgb(${r}, ${g}, ${b})`;
        if (colorCache.size >= MAX_COLOR_CACHE) {
          const firstKey = colorCache.keys().next().value;
          if (firstKey !== undefined) colorCache.delete(firstKey);
        }
        colorCache.set(imageUrl, color);
        resolve(color);
      } catch {
        resolve('');
      }
    };
    img.onerror = () => resolve('');
    img.src = imageUrl;
  });
}
