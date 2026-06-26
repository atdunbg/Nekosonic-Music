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
  return {
    id: song.id,
    name: song.name,
    ar,
    al,
    dt,
    localPath: song.localPath,
    alg: song.alg || undefined,
    br: song.br || undefined,
  };
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
