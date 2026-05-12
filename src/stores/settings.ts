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

interface SettingsData {
  audioQuality: AudioQuality;
  downloadPath: string;
  theme: ThemeMode;
  closeAction: CloseAction;
}

function loadSettings(): SettingsData {
  try {
    const raw = localStorage.getItem('app_settings');
    if (raw) return JSON.parse(raw);
  } catch {}
  return {
    audioQuality: 'standard',
    downloadPath: '',
    theme: 'dark',
    closeAction: 'ask',
  };
}

export const useSettingsStore = defineStore('settings', () => {
  const saved = loadSettings();

  const audioQuality = ref<AudioQuality>(saved.audioQuality);
  const downloadPath = ref<string>(saved.downloadPath);
  const theme = ref<ThemeMode>(saved.theme);
  const closeAction = ref<CloseAction>(saved.closeAction || 'ask');

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

  watch([audioQuality, downloadPath, theme, closeAction], () => {
    const data: SettingsData = {
      audioQuality: audioQuality.value,
      downloadPath: downloadPath.value,
      theme: theme.value,
      closeAction: closeAction.value,
    };
    localStorage.setItem('app_settings', JSON.stringify(data));
  }, { deep: true });

  return {
    audioQuality,
    downloadPath,
    theme,
    closeAction,
    setAudioQuality,
    setDownloadPath,
    setTheme,
    setCloseAction,
  };
});
