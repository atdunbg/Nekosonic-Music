export interface Song {
  id: number;
  name: string;
  ar: { id?: number; name: string }[];
  al: { id?: number; picUrl: string; name?: string };
  dt?: number;
  localPath?: string;
}

export function normalizeSong(song: any): Song {
  const al = {
    id: song.al?.id || song.album?.id,
    picUrl: song.al?.picUrl || song.album?.picUrl || '',
    name: song.al?.name || song.album?.name,
  };
  const ar = (song.ar && song.ar.length > 0) ? song.ar : (song.artists || []);
  let dt = song.dt || song.duration || 0;
  if (dt < 100 || dt > 7200000) dt = 0;
  return {
    id: song.id,
    name: song.name,
    ar,
    al,
    dt,
    localPath: song.localPath,
  };
}

export function getCoverUrl(song: Song | null, sizeParam = ''): string {
  if (!song) return '';
  const raw = song.al?.picUrl || '';
  if (!raw) return '';
  if (!sizeParam || raw.startsWith('data:')) return raw;
  return raw + sizeParam;
}

const colorCache = new Map<string, string>();

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
