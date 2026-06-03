import { defineStore } from 'pinia';
import { ref, watch, computed } from 'vue';

export type AudioQuality = 'standard' | 'higher' | 'exhigh' | 'lossless' | 'hires';
export type ThemeColor = 'blue' | 'green' | 'rose' | 'violet' | 'orange' | 'cyan' | 'pink';
export type Appearance = 'dark' | 'light';
export type CloseAction = 'ask' | 'minimize' | 'exit';

export const themeLabels: Record<ThemeColor, string> = {
  blue: '天蓝',
  green: '翠绿',
  rose: '玫红',
  violet: '紫罗兰',
  orange: '橙色',
  cyan: '青色',
  pink: '粉色',
};

export const themeColors: Record<ThemeColor, string> = {
  blue: '#3b82f6',
  green: '#22c55e',
  rose: '#f43f5e',
  violet: '#8b5cf6',
  orange: '#f97316',
  cyan: '#06b6d4',
  pink: '#ec4899',
};

export const appearanceLabels: Record<Appearance, string> = {
  dark: '深色',
  light: '浅色',
};

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

interface SettingsData {
  audioQuality: AudioQuality;
  downloadPath: string;
  localMusicPaths: string[];
  theme: ThemeColor;
  appearance: Appearance;
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
      const theme = parsed.theme || parsed.accentColor || 'blue';
      const validThemes: ThemeColor[] = ['blue', 'green', 'rose', 'violet', 'orange', 'cyan', 'pink'];
      const validAppearances: Appearance[] = ['dark', 'light'];
      const appearance = validAppearances.includes(parsed.appearance) ? parsed.appearance : 'dark';
      if (parsed.theme && parsed.theme.startsWith('light-')) {
        return {
          audioQuality: parsed.audioQuality || 'standard',
          downloadPath: parsed.downloadPath || '',
          localMusicPaths: parsed.localMusicPaths || [],
          theme: validThemes.includes(parsed.theme.slice(6)) ? parsed.theme.slice(6) : 'blue',
          appearance: 'light',
          closeAction: parsed.closeAction || 'ask',
          shortcuts: { ...defaultShortcuts, ...(parsed.shortcuts || {}) },
          outputDevice: parsed.outputDevice || null,
          volume: typeof parsed.volume === 'number' ? parsed.volume : 100,
        };
      }
      return {
        audioQuality: parsed.audioQuality || 'standard',
        downloadPath: parsed.downloadPath || '',
        localMusicPaths: parsed.localMusicPaths || [],
        theme: validThemes.includes(theme) ? theme : 'blue',
        appearance,
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
    theme: 'blue',
    appearance: 'dark',
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
  const localMusicPaths = ref<string[]>(saved.localMusicPaths);
  const theme = ref<ThemeColor>(saved.theme);
  const appearance = ref<Appearance>(saved.appearance);
  const closeAction = ref<CloseAction>(saved.closeAction || 'ask');
  const shortcuts = ref<Record<string, ShortcutBinding>>(saved.shortcuts);
  const outputDevice = ref<string | null>(saved.outputDevice);
  const volume = ref<number>(saved.volume);

  const dataTheme = computed(() =>
    appearance.value === 'light' ? `light-${theme.value}` : theme.value
  );

  function setAudioQuality(q: AudioQuality) {
    audioQuality.value = q;
  }

  function setDownloadPath(p: string) {
    downloadPath.value = p;
  }

  function addLocalMusicPath(p: string) {
    if (!localMusicPaths.value.includes(p)) {
      localMusicPaths.value = [...localMusicPaths.value, p];
    }
  }

  function removeLocalMusicPath(p: string) {
    localMusicPaths.value = localMusicPaths.value.filter(v => v !== p);
  }

  function setTheme(t: ThemeColor) {
    theme.value = t;
  }

  function setAppearance(a: Appearance) {
    appearance.value = a;
  }

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
    localMusicPaths.value = [];
    theme.value = 'blue';
    appearance.value = 'dark';
    closeAction.value = 'ask';
    shortcuts.value = { ...defaultShortcuts };
    outputDevice.value = null;
    volume.value = 100;
  }

  watch([audioQuality, downloadPath, localMusicPaths, theme, appearance, closeAction, shortcuts, outputDevice, volume], () => {
    const data: SettingsData = {
      audioQuality: audioQuality.value,
      downloadPath: downloadPath.value,
      localMusicPaths: localMusicPaths.value,
      theme: theme.value,
      appearance: appearance.value,
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
    localMusicPaths,
    theme,
    appearance,
    dataTheme,
    closeAction,
    shortcuts,
    outputDevice,
    volume,
    setAudioQuality,
    setDownloadPath,
    addLocalMusicPath,
    removeLocalMusicPath,
    setTheme,
    setAppearance,
    setCloseAction,
    setOutputDevice,
    setShortcut,
    resetShortcuts,
    resetAll,
  };
});
