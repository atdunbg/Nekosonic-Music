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
