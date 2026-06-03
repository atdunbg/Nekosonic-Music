import { MusicApi } from '../api';
import type { Song } from '../utils/song';

export interface LocalSong {
  id: number;
  name: string;
  artist: string;
  album: string;
  duration: number;
  cover: string | null;
  filename: string;
  fileSize: number;
  path: string;
  local: boolean;
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

export function localSongToSong(local: LocalSong): Song {
  return {
    id: local.id,
    name: local.name,
    ar: local.artist ? [{ name: local.artist }] : [],
    al: { picUrl: local.cover || '', name: local.album || undefined },
    dt: local.duration || undefined,
    localPath: local.path,
  };
}

export async function fetchMissingCovers(songs: LocalSong[]): Promise<void> {
  const missing = songs.filter(s => !s.cover && s.id > 0 && s.id < 1e12);
  if (missing.length === 0) return;
  const ids = [...new Set(missing.map(s => s.id))];
  try {
    const jsonStr: string = await MusicApi.getSongDetail(JSON.stringify(ids));
    const data = JSON.parse(jsonStr);
    const detailMap = new Map<number, string>();
    for (const s of data.songs || []) {
      const url = s.al?.picUrl;
      if (url && s.id) detailMap.set(s.id, url + '?param=100y100');
    }
    for (const song of missing) {
      const url = detailMap.get(song.id);
      if (url) song.cover = url;
    }
  } catch { /* 忽略 */ }
}
