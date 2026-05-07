/**
 * 统一规范化歌曲对象，确保 al.picUrl、ar、dt 字段存在且合理
 */
export function normalizeSong(song: any) {
  const normalized = { ...song };
  // 封面 / 艺术家兼容
  if (!normalized.al?.picUrl && normalized.album?.picUrl) {
    normalized.al = { ...normalized.al, picUrl: normalized.album.picUrl };
  }
  if (!normalized.ar || normalized.ar.length === 0) {
    normalized.ar = normalized.artists || [];
  }
  // 时长：只保留合理的 dt（100ms ~ 2小时），否则置 0
  if (!normalized.dt || normalized.dt < 100 || normalized.dt > 7200000) {
    normalized.dt = 0;
  }
  return normalized;
}