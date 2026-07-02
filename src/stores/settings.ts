import { defineStore } from 'pinia';
import { ref, watch, computed } from 'vue';
import { getPresetSkin, isPresetSkinId, applySkinColors, type SkinColors } from '../skins';
import { migrateSettings, CURRENT_SCHEMA_VERSION } from './migrations/settingsMigrations';

export type AudioQuality = 'standard' | 'higher' | 'exhigh' | 'lossless' | 'hires';
export type CloseAction = 'ask' | 'minimize' | 'exit';

export const qualityLabels: Record<AudioQuality, string> = {
  standard: '标准',
  higher: '较高',
  exhigh: '极高 (HQ)',
  lossless: '无损 (SQ)',
  hires: 'Hi-Res',
};

export const closeActionLabels: Record<CloseAction, string> = {
  ask: '每次询问',
  minimize: '最小化到托盘',
  exit: '直接退出',
};

/**
 * 音源类型
 * - builtin: 内置源（bodian/kugou/kuwo/bilibili），由后端实现
 * - custom: 自定义 HTTP API 源，由用户配置模板 URL + JSON 路径
 */
export type MusicSourceKind = 'builtin' | 'custom';

/**
 * 音源配置（前端持久化、调用后端命令都用这个结构，与后端 SourceConfig 对齐）
 */
export interface MusicSourceConfig {
  /** 唯一 ID（builtin: 'bodian'/'kugou'/...；custom: 'custom-xxxx'） */
  id: string;
  /** 显示名称 */
  label: string;
  /** 是否启用 */
  enabled: boolean;
  /** 类型 */
  kind: MusicSourceKind;

  // ===== 自定义 HTTP API 源字段（kind === 'custom' 时使用）=====
  /** 搜索 URL 模板，支持 {keyword} {page} {limit} 占位符 */
  searchUrl?: string;
  /** 获取播放 URL 接口，支持 {id} {quality} 占位符 */
  urlApi?: string;
  /** JSON 路径：搜索结果数组（如 "data.songs"） */
  searchPath?: string;
  /** JSON 路径：URL 字符串字段（如 "data.url"） */
  urlPath?: string;
  /** 字段映射（自定义源 JSON 字段名） */
  idField?: string;
  nameField?: string;
  artistField?: string;
  albumField?: string;
  durationField?: string;
  picField?: string;
}

/**
 * 内置音源默认配置（顺序即默认优先级，波点为首选，参考 SPlayer）
 * kuwo 使用 kwDES + mobi.s 接口，能绕过 VIP 限制播放完整歌曲
 */
export const builtinMusicSourceDefaults: MusicSourceConfig[] = [
  { id: 'bodian', label: '波点音乐', enabled: true, kind: 'builtin' },
  { id: 'kugou', label: '酷狗音乐', enabled: true, kind: 'builtin' },
  { id: 'kuwo', label: '酷我音乐', enabled: true, kind: 'builtin' },
];

/** 内置音源 id 集合，用于判断是否可删除 */
export const builtinSourceIds = new Set(builtinMusicSourceDefaults.map(s => s.id));

export interface ShortcutBinding {
  key: string;
  label: string;
}

export const defaultShortcuts: Record<string, ShortcutBinding> = {
  playPause: { key: 'Control+KeyP', label: '播放/暂停' },
  prev: { key: 'Control+ArrowLeft', label: '上一首' },
  next: { key: 'Control+ArrowRight', label: '下一首' },
  volUp: { key: 'Control+ArrowUp', label: '音量增加' },
  volDown: { key: 'Control+ArrowDown', label: '音量减小' },
  globalPlayPause: { key: 'Control+Alt+KeyP', label: '播放/暂停（全局）' },
  globalPrev: { key: 'Control+Alt+ArrowLeft', label: '上一首（全局）' },
  globalNext: { key: 'Control+Alt+ArrowRight', label: '下一首（全局）' },
  globalVolUp: { key: 'Control+Alt+ArrowUp', label: '音量增加（全局）' },
  globalVolDown: { key: 'Control+Alt+ArrowDown', label: '音量减小（全局）' },
};

export interface CustomSkin {
  id: string;
  name: string;
  preview: string;
  colors: SkinColors;
  /** 壁纸图片路径，为空则使用纯色背景 */
  wallpaper: string;
  /** 壁纸模糊度 0-30 */
  wallpaperBlur: number;
  /** 壁纸透明度 0-1 */
  wallpaperOpacity: number;
}

export interface MusicFolder {
  path: string;
  enabled: boolean;
}

interface SettingsData {
  audioQuality: AudioQuality;
  downloadPath: string;
  localMusicFolders: MusicFolder[];
  skin: string; // 预设皮肤 id 或 custom-xxx
  customSkins: CustomSkin[];
  closeAction: CloseAction;
  shortcuts: Record<string, ShortcutBinding>;
  outputDevice: string | null;
  volume: number;
  /** 第三方音源总开关 */
  musicSourceEnabled: boolean;
  /** 音源列表（有序：搜索聚合顺序 / fallback 优先级；含 builtin + custom） */
  musicSources: MusicSourceConfig[];
  schemaVersion: number;
}

function defaultSettings(): SettingsData {
  return {
    audioQuality: 'standard',
    downloadPath: '',
    localMusicFolders: [],
    skin: 'dark-blue',
    customSkins: [],
    closeAction: 'ask',
    shortcuts: { ...defaultShortcuts },
    outputDevice: null,
    volume: 100,
    musicSourceEnabled: true,
    musicSources: builtinMusicSourceDefaults.map(s => ({ ...s })),
    schemaVersion: CURRENT_SCHEMA_VERSION,
  };
}

function loadSettings(): SettingsData {
  const defaults = defaultSettings();

  try {
    const raw = localStorage.getItem('app_settings');
    if (!raw) return defaults;

    const parsed = JSON.parse(raw);
    const migrated = migrateSettings(parsed);

    return {
      audioQuality: migrated.audioQuality || defaults.audioQuality,
      downloadPath: migrated.downloadPath || defaults.downloadPath,
      localMusicFolders: Array.isArray(migrated.localMusicFolders) ? migrated.localMusicFolders : defaults.localMusicFolders,
      skin: migrated.skin || defaults.skin,
      customSkins: Array.isArray(migrated.customSkins) ? migrated.customSkins : defaults.customSkins,
      closeAction: migrated.closeAction || defaults.closeAction,
      shortcuts: { ...defaultShortcuts, ...(migrated.shortcuts || {}) },
      outputDevice: migrated.outputDevice ?? null,
      volume: typeof migrated.volume === 'number' ? migrated.volume : defaults.volume,
      musicSourceEnabled: typeof migrated.musicSourceEnabled === 'boolean' ? migrated.musicSourceEnabled : defaults.musicSourceEnabled,
      musicSources: Array.isArray(migrated.musicSources) ? migrated.musicSources : defaults.musicSources,
      schemaVersion: CURRENT_SCHEMA_VERSION,
    };
  } catch { /* 忽略 */ }
  return defaults;
}

export const useSettingsStore = defineStore('settings', () => {
  const saved = loadSettings();

  const audioQuality = ref<AudioQuality>(saved.audioQuality);
  const downloadPath = ref<string>(saved.downloadPath);
  const localMusicFolders = ref<MusicFolder[]>(saved.localMusicFolders);
  const skin = ref<string>(saved.skin);
  const customSkins = ref<CustomSkin[]>(saved.customSkins);
  const closeAction = ref<CloseAction>(saved.closeAction || 'ask');
  const shortcuts = ref<Record<string, ShortcutBinding>>(saved.shortcuts);
  const outputDevice = ref<string | null>(saved.outputDevice);
  const volume = ref<number>(saved.volume);
  const musicSourceEnabled = ref<boolean>(saved.musicSourceEnabled);
  const musicSources = ref<MusicSourceConfig[]>(saved.musicSources);

  const isPreset = computed(() => isPresetSkinId(skin.value));

  const currentCustomSkin = computed(() => {
    if (isPreset.value) return null;
    return customSkins.value.find(s => s.id === skin.value) || null;
  });

  const skinPreview = computed(() => {
    if (isPreset.value) {
      return getPresetSkin(skin.value)?.preview || '#3b82f6';
    }
    return currentCustomSkin.value?.preview || '#3b82f6';
  });

  const currentColors = computed<SkinColors>(() => {
    if (isPreset.value) {
      return getPresetSkin(skin.value)!.colors;
    }
    const custom = currentCustomSkin.value;
    if (!custom) {
      return getPresetSkin('dark-blue')!.colors;
    }
    return custom.colors;
  });

  const currentWallpaper = computed(() => {
    if (isPreset.value) return { path: '', blur: 10, opacity: 0.3 };
    const custom = currentCustomSkin.value;
    return {
      path: custom?.wallpaper || '',
      blur: custom?.wallpaperBlur ?? 10,
      opacity: custom?.wallpaperOpacity ?? 0.3,
    };
  });

  function setSkin(id: string) {
    skin.value = id;
  }

  function addCustomSkin(s: CustomSkin) {
    customSkins.value = [...customSkins.value, s];
    skin.value = s.id;
  }

  function updateCustomSkin(id: string, updates: Partial<CustomSkin>) {
    customSkins.value = customSkins.value.map(s =>
      s.id === id ? { ...s, ...updates } : s
    );
    if (skin.value === id) {
      applySkin();
    }
  }

  function removeCustomSkin(id: string) {
    customSkins.value = customSkins.value.filter(s => s.id !== id);
    if (skin.value === id) {
      skin.value = 'dark-blue';
    }
  }

  function applySkin() {
    let colors: SkinColors;
    if (isPreset.value) {
      const preset = getPresetSkin(skin.value);
      colors = preset!.colors;
    } else {
      const custom = currentCustomSkin.value;
      if (!custom) {
        skin.value = 'dark-blue';
        colors = getPresetSkin('dark-blue')!.colors;
      } else {
        colors = custom.colors;
      }
    }
    applySkinColors(colors);
  }

  function setAudioQuality(q: AudioQuality) {
    audioQuality.value = q;
  }

  function setDownloadPath(p: string) {
    downloadPath.value = p;
  }

  function addLocalMusicPath(p: string) {
    if (!localMusicFolders.value.some(f => f.path === p)) {
      localMusicFolders.value = [...localMusicFolders.value, { path: p, enabled: true }];
    }
  }

  function removeLocalMusicPath(p: string) {
    localMusicFolders.value = localMusicFolders.value.filter(f => f.path !== p);
  }

  function toggleLocalMusicFolder(p: string) {
    localMusicFolders.value = localMusicFolders.value.map(f =>
      f.path === p ? { ...f, enabled: !f.enabled } : f
    );
  }

  const enabledMusicPaths = computed(() =>
    localMusicFolders.value.filter(f => f.enabled).map(f => f.path)
  );

  function setCloseAction(a: CloseAction) {
    closeAction.value = a;
  }

  function setShortcut(id: string, key: string) {
    shortcuts.value = { ...shortcuts.value, [id]: { ...shortcuts.value[id], key } };
  }

  function resetShortcuts() {
    shortcuts.value = { ...defaultShortcuts };
  }

  function setOutputDevice(device: string | null) {
    outputDevice.value = device;
  }

  // ===== 音源管理 =====

  function setMusicSourceEnabled(v: boolean) {
    musicSourceEnabled.value = v;
  }

  /** 切换某个音源的启用状态（保持原顺序） */
  function toggleMusicSource(id: string) {
    musicSources.value = musicSources.value.map(s =>
      s.id === id ? { ...s, enabled: !s.enabled } : s
    );
  }

  /** 设置某个音源的 enabled 为指定值 */
  function setMusicSourceEnabledById(id: string, enabled: boolean) {
    musicSources.value = musicSources.value.map(s =>
      s.id === id ? { ...s, enabled } : s
    );
  }

  /** 调整音源顺序（拖拽排序用）：把 from 移到 to 位置 */
  function moveMusicSource(from: number, to: number) {
    if (from < 0 || to < 0 || from >= musicSources.value.length) return;
    const arr = [...musicSources.value];
    const [item] = arr.splice(from, 1);
    arr.splice(Math.min(to, arr.length), 0, item);
    musicSources.value = arr;
  }

  /** 添加自定义音源 */
  function addCustomMusicSource(cfg: MusicSourceConfig) {
    if (musicSources.value.some(s => s.id === cfg.id)) return;
    musicSources.value = [...musicSources.value, cfg];
  }

  /** 更新自定义音源配置 */
  function updateMusicSource(id: string, updates: Partial<MusicSourceConfig>) {
    musicSources.value = musicSources.value.map(s =>
      s.id === id ? { ...s, ...updates, id: s.id, kind: s.kind } : s
    );
  }

  /** 删除音源（内置源不可删除，只能禁用） */
  function removeMusicSource(id: string) {
    if (builtinSourceIds.has(id)) return;
    musicSources.value = musicSources.value.filter(s => s.id !== id);
  }

  /** 恢复默认音源列表（保留用户已有的自定义源） */
  function resetMusicSourcesToDefault() {
    const customs = musicSources.value.filter(s => s.kind === 'custom');
    musicSources.value = [...builtinMusicSourceDefaults.map(s => ({ ...s })), ...customs];
  }

  /** 当前已启用的音源列表（含总开关检查，传给后端搜索/降级用） */
  const enabledMusicSources = computed(() =>
    musicSourceEnabled.value ? musicSources.value.filter(s => s.enabled) : []
  );

  function resetAll() {
    audioQuality.value = 'standard';
    downloadPath.value = '';
    localMusicFolders.value = [];
    skin.value = 'dark-blue';
    customSkins.value = [];
    closeAction.value = 'ask';
    shortcuts.value = { ...defaultShortcuts };
    outputDevice.value = null;
    volume.value = 100;
    musicSourceEnabled.value = true;
    musicSources.value = builtinMusicSourceDefaults.map(s => ({ ...s }));
  }

  watch([audioQuality, downloadPath, localMusicFolders, skin, customSkins, closeAction, shortcuts, outputDevice, volume, musicSourceEnabled, musicSources], () => {
    const data: SettingsData = {
      audioQuality: audioQuality.value,
      downloadPath: downloadPath.value,
      localMusicFolders: localMusicFolders.value,
      skin: skin.value,
      customSkins: customSkins.value,
      closeAction: closeAction.value,
      shortcuts: shortcuts.value,
      outputDevice: outputDevice.value,
      volume: volume.value,
      musicSourceEnabled: musicSourceEnabled.value,
      musicSources: musicSources.value,
      schemaVersion: CURRENT_SCHEMA_VERSION,
    };
    localStorage.setItem('app_settings', JSON.stringify(data));
  }, { deep: true });

  return {
    audioQuality,
    downloadPath,
    localMusicFolders,
    enabledMusicPaths,
    skin,
    customSkins,
    isPreset,
    currentCustomSkin,
    currentColors,
    skinPreview,
    currentWallpaper,
    closeAction,
    shortcuts,
    outputDevice,
    volume,
    musicSourceEnabled,
    musicSources,
    enabledMusicSources,
    setSkin,
    addCustomSkin,
    updateCustomSkin,
    removeCustomSkin,
    applySkin,
    setAudioQuality,
    setDownloadPath,
    addLocalMusicPath,
    removeLocalMusicPath,
    toggleLocalMusicFolder,
    setCloseAction,
    setOutputDevice,
    setMusicSourceEnabled,
    toggleMusicSource,
    setMusicSourceEnabledById,
    moveMusicSource,
    addCustomMusicSource,
    updateMusicSource,
    removeMusicSource,
    resetMusicSourcesToDefault,
    setShortcut,
    resetShortcuts,
    resetAll,
  };
});
