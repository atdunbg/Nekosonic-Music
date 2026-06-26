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
export const CURRENT_SCHEMA_VERSION = 3;

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
