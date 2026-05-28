<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <div class="flex items-center gap-4 mb-6">
      <h1 class="text-2xl font-bold">本地音乐</h1>
      <span v-if="songs.length" class="text-xs text-content-3">{{ songs.length }} 首</span>
      <button
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
      <SongListItem
        v-for="(song, index) in normalizedSongs"
        :key="song.id + '-' + index"
        :song="song"
        :index="index"
        :is-current="player.currentSong?.id === song.id"
        show-index
        show-duration
        show-playing-overlay
        :container-class="player.currentSong?.id === song.id ? 'bg-accent-dim hover:bg-accent-dim' : 'hover:bg-subtle'"
        @click="player.playFromList(normalizedSongs, index)"
      >
        <template #actions>
          <span class="text-xs text-content-3 flex-shrink-0">{{ formatFileSize(songs[index].fileSize) }}</span>
          <div class="relative flex-shrink-0">
            <button
              @click.stop="toggleMenu(songs[index].id)"
              class="text-content-3 hover:text-content transition p-1 rounded hover:bg-muted"
              title="更多"
            >
              <IconEllipsis class="w-4 h-4 fill-current" />
            </button>
            <Transition name="fade">
              <div v-if="openMenuId === songs[index].id" class="absolute right-0 top-full mt-1 w-44 bg-surface border border-line rounded-xl shadow-2xl overflow-hidden z-50" @click.stop>
                <button @click="confirmDelete(songs[index])" class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-danger/80 hover:bg-danger/10 transition">
                  <IconTrash2 style="font-size: 14px" />
                  从磁盘中删除
                </button>
              </div>
            </Transition>
          </div>
        </template>
      </SongListItem>
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
import { ref, computed, onMounted, onActivated, onBeforeUnmount, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import { useSettingsStore } from '../stores/settings';
import { showToast } from '../composables/useToast';
import { pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import SongListItem from '../components/SongListItem.vue';
import IconEllipsis from '~icons/lucide/ellipsis';
import IconTrash2 from '~icons/lucide/trash-2';
import type { Song } from '../utils/song';

defineOptions({ name: 'LocalMusicView' });

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

const normalizedSongs = computed(() => songs.value.map(toSong));

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
  pageCacheInvalidate('localMusic');
  try {
    const list = await invoke<LocalSong[]>('list_local_songs', { downloadPath: settings.downloadPath || null });
    songs.value = list;
    pageCacheSet('localMusic', list);
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
  } catch { /* 忽略 */ }
}

onMounted(refresh);

onActivated(() => {
  if (pageCacheIsStale('localMusic')) refresh();
});

watch(() => settings.downloadPath, () => { refresh(); });

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

function toSong(local: LocalSong): Song {
  return {
    id: local.id,
    name: local.name,
    ar: local.artist ? [{ name: local.artist }] : [],
    al: { picUrl: local.cover || '', name: local.album || undefined },
    dt: local.duration || undefined,
    localPath: local.path,
  };
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
