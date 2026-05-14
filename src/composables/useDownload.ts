import { reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useSettingsStore } from '../stores/settings';
import { showToast } from '../composables/useToast';

interface DownloadTask {
  id: number;
  name: string;
  progress: number;
}

const downloadingIds = reactive<Set<number>>(new Set());
const tasks = reactive<DownloadTask[]>([]);
const localSongIds = reactive<Set<number>>(new Set());

let listenerSetup = false;
let storeSetup = false;

async function setupDownloadListener() {
  if (listenerSetup) return;
  listenerSetup = true;
  await listen<{ id: number; progress: number; name: string }>('download-progress', (event) => {
    const { id, progress, name } = event.payload;
    if (progress >= 100) {
      const idx = tasks.findIndex(t => t.id === id);
      if (idx >= 0) {
        tasks.splice(idx, 1);
        downloadingIds.delete(id);
        showToast(`${name} 下载完成`, 'success');
      }
    } else {
      const task = tasks.find(t => t.id === id);
      if (task) {
        task.progress = progress;
      }
    }
  });
}

async function refreshLocalIds() {
  try {
    const settings = useSettingsStore();
    const list: { id: number }[] = await invoke('list_local_songs', { downloadPath: settings.downloadPath || null });
    localSongIds.clear();
    for (const s of list) {
      localSongIds.add(s.id);
    }
  } catch {}
}

function ensureStoreSetup() {
  if (storeSetup) return;
  storeSetup = true;
  const settings = useSettingsStore();
  refreshLocalIds();
  watch(() => settings.downloadPath, () => {
    refreshLocalIds();
  });
}

function isDownloaded(songId: number): boolean {
  return localSongIds.has(songId);
}

function isDownloading(songId: number): boolean {
  return downloadingIds.has(songId);
}

function getDownloadProgress(songId: number): number {
  const task = tasks.find(t => t.id === songId);
  return task?.progress ?? 0;
}

async function downloadSong(song: { id: number; name: string; ar?: { name: string }[]; artists?: { name: string }[]; al?: { picUrl?: string; name?: string }; album?: { picUrl?: string; name?: string }; dt?: number; duration?: number }) {
  if (downloadingIds.has(song.id)) return;
  if (localSongIds.has(song.id)) {
    showToast(`${song.name} 已下载`, 'info');
    return;
  }

  const settings = useSettingsStore();
  const artist = song.ar?.map(a => a.name).join(' / ') || song.artists?.map(a => a.name).join(' / ') || '未知';
  const albumName = song.al?.name || song.album?.name || null;
  const durationVal = song.dt || song.duration || null;
  const coverUrl = song.al?.picUrl || song.album?.picUrl || null;

  downloadingIds.add(song.id);
  tasks.push({ id: song.id, name: song.name, progress: 0 });

  try {
    await invoke('download_song', {
      query: {
        id: song.id,
        name: song.name,
        artist,
        album: albumName,
        duration: durationVal,
        coverUrl,
        level: settings.audioQuality,
        downloadPath: settings.downloadPath || null,
      },
    });
    localSongIds.add(song.id);
  } catch (e: any) {
    downloadingIds.delete(song.id);
    const idx = tasks.findIndex(t => t.id === song.id);
    if (idx >= 0) tasks.splice(idx, 1);
    if (e === '文件已存在') {
      localSongIds.add(song.id);
      showToast(`${song.name} 已下载`, 'info');
    } else if (e === 'VIP歌曲无法下载') {
      showToast(`${song.name} 为 VIP 歌曲，无法下载`, 'error');
    } else if (typeof e === 'string' && e.includes('VIP')) {
      showToast(`${song.name} 需要 VIP 权限才能下载`, 'error');
    } else {
      showToast(`下载失败: ${e}`, 'error');
    }
  }
}

export function useDownload() {
  setupDownloadListener();
  ensureStoreSetup();
  return {
    downloadingIds,
    tasks,
    localSongIds,
    isDownloaded,
    isDownloading,
    getDownloadProgress,
    downloadSong,
    refreshLocalIds,
  };
}
