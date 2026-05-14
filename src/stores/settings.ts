import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export type AudioQuality = 'standard' | 'higher' | 'exhigh' | 'lossless' | 'hires';
export type ThemeMode = 'dark' | 'light';
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
  prev: { key: 'Control+ArrowLeft', label: '上一首' },
  next: { key: 'Control+ArrowRight', label: '下一首' },
  volUp: { key: 'Control+ArrowUp', label: '音量增加' },
  volDown: { key: 'Control+ArrowDown', label: '音量减小' },
  globalPrev: { key: 'Alt+Control+ArrowLeft', label: '上一首（全局）' },
  globalNext: { key: 'Alt+Control+ArrowRight', label: '下一首（全局）' },
  globalVolUp: { key: 'Alt+Control+ArrowUp', label: '音量增加（全局）' },
  globalVolDown: { key: 'Alt+Control+ArrowDown', label: '音量减小（全局）' },
};

interface SettingsData {
  audioQuality: AudioQuality;
  downloadPath: string;
  theme: ThemeMode;
  closeAction: CloseAction;
  shortcuts: Record<string, ShortcutBinding>;
}

function loadSettings(): SettingsData {
  try {
    const raw = localStorage.getItem('app_settings');
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        audioQuality: parsed.audioQuality || 'standard',
        downloadPath: parsed.downloadPath || '',
        theme: parsed.theme || 'dark',
        closeAction: parsed.closeAction || 'ask',
        shortcuts: { ...defaultShortcuts, ...(parsed.shortcuts || {}) },
      };
    }
  } catch {}
  return {
    audioQuality: 'standard',
    downloadPath: '',
    theme: 'dark',
    closeAction: 'ask',
    shortcuts: { ...defaultShortcuts },
  };
}

export const useSettingsStore = defineStore('settings', () => {
  const saved = loadSettings();

  const audioQuality = ref<AudioQuality>(saved.audioQuality);
  const downloadPath = ref<string>(saved.downloadPath);
  const theme = ref<ThemeMode>(saved.theme);
  const closeAction = ref<CloseAction>(saved.closeAction || 'ask');
  const shortcuts = ref<Record<string, ShortcutBinding>>(saved.shortcuts);

  function setAudioQuality(q: AudioQuality) {
    audioQuality.value = q;
  }

  function setDownloadPath(p: string) {
    downloadPath.value = p;
  }

  function setTheme(t: ThemeMode) {
    theme.value = t;
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

  function resetAll() {
    audioQuality.value = 'standard';
    downloadPath.value = '';
    theme.value = 'dark';
    closeAction.value = 'ask';
    shortcuts.value = { ...defaultShortcuts };
  }

  watch([audioQuality, downloadPath, theme, closeAction, shortcuts], () => {
    const data: SettingsData = {
      audioQuality: audioQuality.value,
      downloadPath: downloadPath.value,
      theme: theme.value,
      closeAction: closeAction.value,
      shortcuts: shortcuts.value,
    };
    localStorage.setItem('app_settings', JSON.stringify(data));
  }, { deep: true });

  return {
    audioQuality,
    downloadPath,
    theme,
    closeAction,
    shortcuts,
    setAudioQuality,
    setDownloadPath,
    setTheme,
    setCloseAction,
    setShortcut,
    resetShortcuts,
    resetAll,
  };
});
