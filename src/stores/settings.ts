import { defineStore } from 'pinia';
import { ref, watch, computed } from 'vue';
import { getPresetSkin, isPresetSkinId, applySkinColors, type SkinColors } from '../skins';

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
  localMusicPaths: string[]; // 旧格式，迁移用
  localMusicFolders: MusicFolder[];
  skin: string; // 预设皮肤 id 或 custom-xxx
  customSkins: CustomSkin[];
  closeAction: CloseAction;
  shortcuts: Record<string, ShortcutBinding>;
  outputDevice: string | null;
  volume: number;
}

function loadSettings(): SettingsData {
  try {
    const raw = localStorage.getItem('app_settings');
    if (raw) {
      const parsed = JSON.parse(raw);

      // 迁移旧版 theme + appearance → skin
      let skin = parsed.skin || 'dark-blue';
      if (!parsed.skin && (parsed.theme || parsed.appearance)) {
        const appearance = parsed.appearance || 'dark';
        const theme = parsed.theme || 'blue';
        const validThemes = ['blue', 'green', 'rose', 'violet', 'orange', 'cyan', 'pink'];
        const t = validThemes.includes(theme) ? theme : 'blue';
        skin = appearance === 'light' ? `light-${t}` : `dark-${t}`;
      }

      // 迁移旧版全局壁纸 → 移入自定义皮肤（如果有自定义皮肤且没有壁纸）
      let customSkins = parsed.customSkins || [];
      if (parsed.wallpaper && customSkins.length > 0) {
        customSkins = customSkins.map((s: any) => {
          if (!s.wallpaper) {
            return { ...s, wallpaper: parsed.wallpaper, wallpaperBlur: parsed.wallpaperBlur ?? 10, wallpaperOpacity: parsed.wallpaperOpacity ?? 0.3 };
          }
          return s;
        });
      }

      // 迁移旧格式 localMusicPaths → localMusicFolders
      let folders: MusicFolder[] = (parsed.localMusicFolders || []).map((f: any) =>
        typeof f === 'string' ? { path: f, enabled: true } : f
      );
      if (!parsed.localMusicFolders && parsed.localMusicPaths?.length) {
        folders = parsed.localMusicPaths.map((p: string) => ({ path: p, enabled: true }));
      }

      return {
        audioQuality: parsed.audioQuality || 'standard',
        downloadPath: parsed.downloadPath || '',
        localMusicPaths: [],
        localMusicFolders: folders,
        skin,
        customSkins,
        closeAction: parsed.closeAction || 'ask',
        shortcuts: { ...defaultShortcuts, ...(parsed.shortcuts || {}) },
        outputDevice: parsed.outputDevice || null,
        volume: typeof parsed.volume === 'number' ? parsed.volume : 100,
      };
    }
  } catch { /* 忽略 */ }
  return {
    audioQuality: 'standard',
    downloadPath: '',
    localMusicPaths: [],
    localMusicFolders: [],
    skin: 'dark-blue',
    customSkins: [],
    closeAction: 'ask',
    shortcuts: { ...defaultShortcuts },
    outputDevice: null,
    volume: 100,
  };
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

  /** 当前皮肤是否为预设皮肤 */
  const isPreset = computed(() => isPresetSkinId(skin.value));

  /** 获取当前自定义皮肤 */
  const currentCustomSkin = computed(() => {
    if (isPreset.value) return null;
    return customSkins.value.find(s => s.id === skin.value) || null;
  });

  /** 获取当前皮肤的预览色 */
  const skinPreview = computed(() => {
    if (isPreset.value) {
      return getPresetSkin(skin.value)?.preview || '#3b82f6';
    }
    return currentCustomSkin.value?.preview || '#3b82f6';
  });

  /** 获取当前皮肤的完整颜色集（响应式） */
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

  /** 获取当前皮肤的壁纸信息 */
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
    // 如果正在使用该皮肤，立即刷新 CSS 变量
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

  /** 应用当前皮肤到 DOM（统一通过 JS 设置 CSS 变量） */
  function applySkin() {
    let colors: SkinColors;
    if (isPreset.value) {
      const preset = getPresetSkin(skin.value);
      colors = preset!.colors;
    } else {
      const custom = currentCustomSkin.value;
      if (!custom) {
        // 找不到自定义皮肤，回退到默认
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

  /** 已启用的扫描路径 */
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
  }

  watch([audioQuality, downloadPath, localMusicFolders, skin, customSkins, closeAction, shortcuts, outputDevice, volume], () => {
    const data: SettingsData = {
      audioQuality: audioQuality.value,
      downloadPath: downloadPath.value,
      localMusicPaths: [],
      localMusicFolders: localMusicFolders.value,
      skin: skin.value,
      customSkins: customSkins.value,
      closeAction: closeAction.value,
      shortcuts: shortcuts.value,
      outputDevice: outputDevice.value,
      volume: volume.value,
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
    setShortcut,
    resetShortcuts,
    resetAll,
  };
});
