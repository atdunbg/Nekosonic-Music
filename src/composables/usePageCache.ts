/**
 * 页面数据缓存（L1 内存 + L2 localStorage 持久化）
 *
 * 两级缓存策略：
 * - L1：内存 Map，访问最快，重启即失
 * - L2：localStorage，持久化，重启后仍可命中
 *
 * 读取顺序：L1 → L2 → miss（L2 命中时回填 L1）
 * 写入策略：同时写 L1 + L2（L2 配额不足时静默淘汰最旧条目）
 */
const TTL = 30 * 60 * 1000;
const MAX_ENTRIES = 30;
const LS_PREFIX = 'pc:';

// L1 内存缓存
const cache = new Map<string, { data: any; ts: number }>();

/** 从 localStorage 读取并校验 TTL */
function lsGet(key: string): { data: any; ts: number } | null {
  try {
    const raw = localStorage.getItem(LS_PREFIX + key);
    if (!raw) return null;
    const entry = JSON.parse(raw);
    if (Date.now() - entry.ts > TTL) {
      localStorage.removeItem(LS_PREFIX + key);
      return null;
    }
    return entry;
  } catch {
    return null;
  }
}

/** 写入 localStorage，配额不足时按 FIFO 淘汰旧条目后重试 */
function lsSet(key: string, entry: { data: any; ts: number }) {
  try {
    localStorage.setItem(LS_PREFIX + key, JSON.stringify(entry));
  } catch {
    // 配额不足：扫描淘汰最旧的条目后重试一次
    const keys: { k: string; ts: number }[] = [];
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i);
      if (k && k.startsWith(LS_PREFIX)) {
        try {
          const e = JSON.parse(localStorage.getItem(k) || '{}');
          keys.push({ k, ts: e.ts || 0 });
        } catch { /* 忽略损坏条目 */ }
      }
    }
    keys.sort((a, b) => a.ts - b.ts);
    // 淘汰最旧的 1/3
    const removeCount = Math.max(1, Math.floor(keys.length / 3));
    for (let i = 0; i < removeCount && i < keys.length; i++) {
      localStorage.removeItem(keys[i].k);
    }
    try {
      localStorage.setItem(LS_PREFIX + key, JSON.stringify(entry));
    } catch { /* 仍然失败则放弃持久化，仅保留内存缓存 */ }
  }
}

function lsDelete(key: string) {
  localStorage.removeItem(LS_PREFIX + key);
}

/** L1 淘汰最旧条目 */
function evictL1IfNeeded() {
  if (cache.size >= MAX_ENTRIES) {
    const firstKey = cache.keys().next().value;
    if (firstKey !== undefined) cache.delete(firstKey);
  }
}

export function pageCacheGet(key: string): any | null {
  // L1
  const l1 = cache.get(key);
  if (l1) {
    if (Date.now() - l1.ts > TTL) {
      cache.delete(key);
      lsDelete(key);
      return null;
    }
    return l1.data;
  }
  // L2
  const l2 = lsGet(key);
  if (l2) {
    // 回填 L1
    evictL1IfNeeded();
    cache.set(key, l2);
    return l2.data;
  }
  return null;
}

export function pageCacheSet(key: string, data: any) {
  const entry = { data, ts: Date.now() };
  evictL1IfNeeded();
  cache.set(key, entry);
  lsSet(key, entry);
}

export function pageCacheInvalidate(key: string) {
  cache.delete(key);
  lsDelete(key);
}

export function pageCacheIsStale(key: string): boolean {
  const entry = cache.get(key) || lsGet(key);
  if (!entry) return true;
  return Date.now() - entry.ts > TTL;
}
