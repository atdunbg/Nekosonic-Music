const cache = new Map<string, { data: any; ts: number }>();
const TTL = 30 * 60 * 1000;
const MAX_ENTRIES = 30;

export function pageCacheGet(key: string): any | null {
  const entry = cache.get(key);
  if (!entry) return null;
  if (Date.now() - entry.ts > TTL) {
    cache.delete(key);
    return null;
  }
  return entry.data;
}

export function pageCacheSet(key: string, data: any) {
  if (cache.size >= MAX_ENTRIES && !cache.has(key)) {
    // 淘汰最旧的条目
    const firstKey = cache.keys().next().value;
    if (firstKey !== undefined) cache.delete(firstKey);
  }
  cache.set(key, { data, ts: Date.now() });
}

export function pageCacheDelete(key: string) {
  cache.delete(key);
}

export function pageCacheInvalidate(key: string) {
  cache.delete(key);
}

export function pageCacheIsStale(key: string): boolean {
  const entry = cache.get(key);
  if (!entry) return true;
  return Date.now() - entry.ts > TTL;
}
