export function formatDuration(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

export function formatTime(sec: number): string {
  if (!sec || isNaN(sec)) return '0:00';
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

const YI = 100_000_000;
const WAN = 10_000;

export function formatPlayCount(count: number): string {
  if (!count) return '0';
  if (count >= YI) return (count / YI).toFixed(1) + '亿';
  if (count >= WAN) return (count / WAN).toFixed(1) + '万';
  return count.toString();
}

export function formatDate(ts: number): string {
  if (!ts) return '';
  const d = new Date(ts);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}
