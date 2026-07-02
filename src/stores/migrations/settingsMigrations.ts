/**
 * 设置数据 Schema 迁移
 *
 * 每次新增字段或修改结构时，新增一个迁移函数并递增 CURRENT_SCHEMA_VERSION。
 * 迁移按版本号顺序执行：v0 → v1 → v2 → ... → CURRENT。
 *
 * 迁移函数签名：(data: any) => any
 * - 输入：旧版数据（可能是任意历史版本）
 * - 输出：升级后的数据
 * - 必须幂等：对已经是目标版本的数据执行应无副作用
 */

/** 当前 Schema 版本号 */
export const CURRENT_SCHEMA_VERSION = 7;

/** 迁移函数按目标版本号索引（data 升级到此版本后即为 vN） */
const migrations: Record<number, (data: any) => any> = {
  /**
   * v1: theme + appearance → skin
   *
   * 旧版用独立的 theme（blue/green/...）和 appearance（dark/light）字段，
   * 合并为统一的 skin 字段（如 "dark-blue"）。
   */
  1: (data) => {
    if (!data.skin && (data.theme || data.appearance)) {
      const appearance = data.appearance || 'dark';
      const theme = data.theme || 'blue';
      const validThemes = ['blue', 'green', 'rose', 'violet', 'orange', 'cyan', 'pink'];
      const t = validThemes.includes(theme) ? theme : 'blue';
      data.skin = appearance === 'light' ? `light-${t}` : `dark-${t}`;
    }
    if (!data.skin) data.skin = 'dark-blue';
    delete data.theme;
    delete data.appearance;
    return data;
  },

  /**
   * v2: 全局 wallpaper → 移入自定义皮肤
   *
   * 旧版有全局 wallpaper/wallpaperBlur/wallpaperOpacity 字段，
   * 迁移到每个自定义皮肤的对应字段（仅填充缺失项）。
   */
  2: (data) => {
    if (data.wallpaper && Array.isArray(data.customSkins) && data.customSkins.length > 0) {
      data.customSkins = data.customSkins.map((s: any) => {
        if (!s.wallpaper) {
          return {
            ...s,
            wallpaper: data.wallpaper,
            wallpaperBlur: s.wallpaperBlur ?? data.wallpaperBlur ?? 10,
            wallpaperOpacity: s.wallpaperOpacity ?? data.wallpaperOpacity ?? 0.3,
          };
        }
        return s;
      });
    }
    delete data.wallpaper;
    delete data.wallpaperBlur;
    delete data.wallpaperOpacity;
    return data;
  },

  /**
   * v3: localMusicPaths (string[]) → localMusicFolders ({path, enabled}[])
   *
   * 旧版本地音乐路径是字符串数组，新版改为带 enabled 开关的对象数组。
   */
  3: (data) => {
    if (!data.localMusicFolders && Array.isArray(data.localMusicPaths) && data.localMusicPaths.length) {
      data.localMusicFolders = data.localMusicPaths.map((p: string) => ({ path: p, enabled: true }));
    }
    if (!Array.isArray(data.localMusicFolders)) {
      data.localMusicFolders = [];
    }
    // 兼容 localMusicFolders 内仍是字符串的情况
    data.localMusicFolders = data.localMusicFolders.map((f: any) =>
      typeof f === 'string' ? { path: f, enabled: true } : f
    );
    delete data.localMusicPaths;
    return data;
  },

  /**
   * v4: 新增第三方音源 fallback 设置
   *
   * 老用户没有 musicSourceEnabled / musicSources 字段，
   * 默认开启总开关，启用 kugou + kuwo（与默认值一致）。
   */
  4: (data) => {
    if (typeof data.musicSourceEnabled !== 'boolean') {
      data.musicSourceEnabled = true;
    }
    if (!Array.isArray(data.musicSources)) {
      data.musicSources = ['kugou', 'kuwo'];
    }
    return data;
  },

  /**
   * v5: 音源模型重构
   *
   * 旧版 musicSources 是 string[]（仅存音源 id），
   * 新版改为 MusicSourceConfig[]（含 id/label/enabled/kind 及自定义 HTTP API 字段）。
   *
   * 迁移规则：
   * - 已是对象数组：保留，补齐缺失字段
   * - 是字符串数组：转换为对象，旧列表里的视为 enabled，bodain（新增波点）默认启用并置于首位
   * - 不存在或格式不对：使用默认内置音源列表
   */
  5: (data) => {
    // 默认内置音源列表（与 settings.ts 的 builtinMusicSourceDefaults 一致）
    const BUILTIN_DEFAULTS: Array<{ id: string; label: string; enabled: boolean; kind: 'builtin' }> = [
      { id: 'bodian', label: '波点音乐', enabled: true, kind: 'builtin' },
      { id: 'kugou', label: '酷狗音乐', enabled: true, kind: 'builtin' },
      { id: 'kuwo', label: '酷我音乐', enabled: true, kind: 'builtin' },
      { id: 'bilibili', label: 'Bilibili 音频', enabled: false, kind: 'builtin' },
    ];
    const BUILTIN_LABELS: Record<string, string> = {
      bodian: '波点音乐',
      kugou: '酷狗音乐',
      kuwo: '酷我音乐',
      bilibili: 'Bilibili 音频',
    };

    const oldSources = data.musicSources;
    // 字符串数组 → 对象数组
    if (Array.isArray(oldSources) && oldSources.length > 0 && typeof oldSources[0] === 'string') {
      const oldIds = new Set(oldSources.filter((s: any) => typeof s === 'string'));
      // 波点（bodian）作为新默认首选，老用户没有则补上启用
      const result: any[] = [];
      for (const def of BUILTIN_DEFAULTS) {
        // 老用户已启用的源保持启用；新增的 bodian 默认启用
        const enabled = oldIds.has(def.id) || def.id === 'bodian' ? true : def.enabled;
        result.push({ ...def, enabled });
      }
      // 老用户列表里可能有自定义源 id（理论上 v4 没有自定义源），保留为 builtin 兜底
      for (const sid of oldIds) {
        if (typeof sid !== 'string') continue;
        if (result.some(r => r.id === sid)) continue;
        result.push({
          id: sid,
          label: BUILTIN_LABELS[sid] || sid,
          enabled: true,
          kind: 'builtin',
        });
      }
      data.musicSources = result;
    } else if (Array.isArray(oldSources) && oldSources.length > 0 && typeof oldSources[0] === 'object') {
      // 已是对象数组：补齐缺失字段（kind / 默认 label 等）
      data.musicSources = oldSources.map((s: any) => {
        if (!s || typeof s !== 'object') return null;
        const id: string = String(s.id || '');
        const kind: string = s.kind === 'custom' ? 'custom' : 'builtin';
        return {
          id,
          label: s.label || BUILTIN_LABELS[id] || id,
          enabled: typeof s.enabled === 'boolean' ? s.enabled : true,
          kind,
          ...(kind === 'custom' ? {
            searchUrl: s.searchUrl,
            urlApi: s.urlApi,
            searchPath: s.searchPath,
            urlPath: s.urlPath,
            idField: s.idField,
            nameField: s.nameField,
            artistField: s.artistField,
            albumField: s.albumField,
            durationField: s.durationField,
            picField: s.picField,
          } : {}),
        };
      }).filter(Boolean);
      // 老用户对象数组里可能没有 bodian，补上
      if (!data.musicSources.some((s: any) => s.id === 'bodian')) {
        data.musicSources.unshift({ id: 'bodian', label: '波点音乐', enabled: true, kind: 'builtin' });
      }
    } else {
      // 不存在或空：使用默认
      data.musicSources = BUILTIN_DEFAULTS.map(s => ({ ...s }));
    }
    return data;
  },

  /**
   * v6: 移除已废弃的 bilibili 内置源
   *
   * bilibili 接口需 cookie，无 cookie 返回 412，已从内置源列表中删除。
   * 注意：kuwo 之前因 r.s 接口返回单引号 JSON 解析失败也被移除，
   * 但现已用 kwDES + mobi.s 接口重写（能绕过 VIP 限制），不再废弃。
   * 用户如果列表里有 bilibili id，直接过滤掉；自定义源保留。
   */
  6: (data) => {
    if (Array.isArray(data.musicSources)) {
      data.musicSources = data.musicSources.filter((s: any) =>
        s && typeof s === 'object' && !(s.kind === 'builtin' && s.id === 'bilibili')
      );
    }
    return data;
  },

  /**
   * v7: 恢复 kuwo 内置源
   *
   * 之前 v6 错误地把 kuwo 也一并移除了。kuwo 现已用 kwDES 加密 + mobi.s 接口重写，
   * 能绕过 VIP 限制播放完整歌曲（参考 SPlayer 实现），是播放 VIP 歌曲的关键音源。
   * 对已升级到 v6（kuwo 被删除）的用户，把 kuwo 加回列表末尾（默认启用）。
   * 已有 kuwo 的用户不受影响（幂等）。
   */
  7: (data) => {
    if (Array.isArray(data.musicSources)) {
      const hasKuwo = data.musicSources.some((s: any) =>
        s && typeof s === 'object' && s.id === 'kuwo' && s.kind === 'builtin'
      );
      if (!hasKuwo) {
        data.musicSources.push({
          id: 'kuwo',
          label: '酷我音乐',
          enabled: true,
          kind: 'builtin',
        });
      }
    }
    return data;
  },
};

/**
 * 执行迁移：将旧版数据升级到 CURRENT_SCHEMA_VERSION
 *
 * @param raw 从 localStorage 读取的原始数据（可能无 schemaVersion）
 * @returns 迁移后的数据，含 schemaVersion = CURRENT_SCHEMA_VERSION
 */
export function migrateSettings(raw: any): any {
  let data = { ...raw };
  let version = typeof data.schemaVersion === 'number' ? data.schemaVersion : 0;

  while (version < CURRENT_SCHEMA_VERSION) {
    const next = version + 1;
    const migrator = migrations[next];
    if (migrator) {
      data = migrator(data);
    }
    version = next;
  }

  data.schemaVersion = CURRENT_SCHEMA_VERSION;
  return data;
}
