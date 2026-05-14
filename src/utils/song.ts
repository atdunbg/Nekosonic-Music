/**
 * 统一规范化歌曲对象，确保 al.picUrl、ar、dt 字段存在且合理
 */
export function normalizeSong(song: any) {
  const normalized = { ...song };
  if (!normalized.al?.picUrl && normalized.album?.picUrl) {
    normalized.al = { ...normalized.al, picUrl: normalized.album.picUrl };
  }
  if (!normalized.al?.name && normalized.album?.name) {
    normalized.al = { ...normalized.al, name: normalized.album.name };
  }
  if (!normalized.ar || normalized.ar.length === 0) {
    normalized.ar = normalized.artists || [];
  }
  if (!normalized.dt || normalized.dt < 100 || normalized.dt > 7200000) {
    normalized.dt = 0;
  }
  return normalized;
}