const cache = new Map<string, { data: any; ts: number }>();
const TTL = 5 * 60 * 1000;

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
