<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <div class="flex items-center gap-4 mb-6">
      <h1 class="text-2xl font-bold">本地音乐</h1>
      <span v-if="songs.length" class="text-xs text-content-3">{{ songs.length }} 首</span>
      <button
        v-if="songs.length"
        @click="refresh"
        class="px-3 py-1 bg-muted hover:bg-emphasis rounded-full text-xs transition"
      >
        刷新
      </button>
    </div>
    <div v-if="loading" class="text-content-2">加载中...</div>
    <div v-else-if="songs.length === 0" class="text-content-3">
      当前文件夹下没有音乐文件，支持 mp3、flac、wav、ogg、aac、m4a 格式
    </div>
    <div v-else class="space-y-2">
      <div
        v-for="(song, index) in songs"
        :key="song.id + '-' + index"
        @click="playLocalSong(song, index)"
        class="flex items-center gap-4 p-3 rounded-xl hover:bg-subtle transition cursor-pointer"
        :class="{ 'bg-subtle': player.currentSong?.id === song.id }"
      >
        <span class="text-xs text-content-3 w-6 text-right flex-shrink-0">{{ index + 1 }}</span>
        <div class="w-10 h-10 rounded-lg overflow-hidden flex-shrink-0 bg-muted">
          <img v-if="song.cover" :src="song.cover" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full flex items-center justify-center">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-content-3"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
          </div>
        </div>
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">{{ song.name }}</p>
          <p class="text-xs text-content-2 truncate">
            {{ song.artist }}<template v-if="song.album"> · {{ song.album }}</template>
          </p>
        </div>
        <span class="text-xs text-content-3 flex-shrink-0">{{ formatDuration(song.duration) }}</span>
        <span class="text-xs text-content-3 flex-shrink-0">{{ formatFileSize(song.fileSize) }}</span>
        <div class="relative flex-shrink-0">
          <button
            @click.stop="toggleMenu(song.id)"
            class="text-content-3 hover:text-content transition p-1 rounded hover:bg-muted"
            title="更多"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/></svg>
          </button>
          <Transition name="fade">
            <div v-if="openMenuId === song.id" class="absolute right-0 top-full mt-1 w-44 bg-surface border border-line rounded-xl shadow-2xl overflow-hidden z-50" @click.stop>
              <button @click="confirmDelete(song)" class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-danger/80 hover:bg-danger/10 transition">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
                从磁盘中删除
              </button>
            </div>
          </Transition>
        </div>
      </div>
    </div>

    <Transition name="fade">
      <div v-if="showDeleteConfirm" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showDeleteConfirm = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-6 select-auto">
          <h2 class="text-lg font-semibold text-content mb-1">确认删除</h2>
          <p class="text-sm text-content-2 mb-5">确定要删除「{{ deleteTarget?.name }}」吗？此操作不可撤销。</p>
          <div class="flex gap-3">
            <button @click="showDeleteConfirm = false"
              class="flex-1 py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">
              取消
            </button>
            <button @click="doDelete"
              class="flex-1 py-2 rounded-lg bg-danger/20 hover:bg-danger/30 text-danger text-sm font-medium transition">
              删除
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore, type Song } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import { useSettingsStore } from '../stores/settings';
import { showToast } from '../composables/useToast';

const player = usePlayerStore();
const download = useDownload();
const settings = useSettingsStore();

interface LocalSong {
  id: number;
  name: string;
  artist: string;
  album: string;
  duration: number;
  cover: string | null;
  filename: string;
  fileSize: number;
  path: string;
  local: boolean;
}

const songs = ref<LocalSong[]>([]);
const loading = ref(true);
const showDeleteConfirm = ref(false);
const deleteTarget = ref<LocalSong | null>(null);
const openMenuId = ref<number | null>(null);

function toggleMenu(id: number) {
  openMenuId.value = openMenuId.value === id ? null : id;
}

function closeMenu() {
  openMenuId.value = null;
}

onMounted(() => { document.addEventListener('click', closeMenu); });
onBeforeUnmount(() => { document.removeEventListener('click', closeMenu); });

async function refresh() {
  loading.value = true;
  try {
    const list = await invoke<LocalSong[]>('list_local_songs', { downloadPath: settings.downloadPath || null });
    songs.value = list;
    fetchMissingCovers();
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

async function fetchMissingCovers() {
  const missing = songs.value.filter(s => !s.cover && s.id > 0 && s.id < 1e12);
  if (missing.length === 0) return;
  const ids = [...new Set(missing.map(s => s.id))];
  try {
    const jsonStr: string = await invoke('get_song_detail', { id: JSON.stringify(ids) });
    const data = JSON.parse(jsonStr);
    const detailMap = new Map<number, string>();
    for (const s of data.songs || []) {
      const url = s.al?.picUrl;
      if (url && s.id) detailMap.set(s.id, url + '?param=100y100');
    }
    for (const song of missing) {
      const url = detailMap.get(song.id);
      if (url) song.cover = url;
    }
  } catch {}
}

onMounted(refresh);

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

function formatDuration(ms: number): string {
  if (!ms || ms === 0) return '--:--';
  const totalSec = Math.floor(ms / 1000);
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  return `${min}:${sec.toString().padStart(2, '0')}`;
}

function toSong(local: LocalSong): Song {
  return {
    id: local.id,
    name: local.name,
    ar: local.artist ? [{ name: local.artist }] : [],
    al: { picUrl: local.cover || '', name: local.album || undefined },
    dt: local.duration || undefined,
    artists: local.artist ? [{ name: local.artist }] : [],
    album: { picUrl: local.cover || undefined, name: local.album || undefined },
    duration: local.duration || undefined,
    localPath: local.path,
  };
}

async function playLocalSong(_song: LocalSong, index: number) {
  const normalized = songs.value.map(s => toSong(s));
  player.playFromList(normalized, index);
}

function confirmDelete(song: LocalSong) {
  openMenuId.value = null;
  deleteTarget.value = song;
  showDeleteConfirm.value = true;
}

async function doDelete() {
  if (!deleteTarget.value) return;
  try {
    await invoke('delete_local_song', { query: { id: deleteTarget.value.id, filename: deleteTarget.value.filename, downloadPath: settings.downloadPath || null } });
    songs.value = songs.value.filter(s => s.id !== deleteTarget.value!.id);
    download.localSongIds.delete(deleteTarget.value.id);
    showToast('已删除', 'success');
  } catch (e) {
    showToast('删除失败', 'error');
  }
  showDeleteConfirm.value = false;
  deleteTarget.value = null;
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
