<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">音乐云盘</h1>
      <span v-if="totalCount" class="text-xs text-content-3">{{ totalCount }} 首</span>
      <template #actions>
        <button
          @click="refresh"
          class="px-3 py-1 bg-muted hover:bg-emphasis rounded-full text-xs transition"
        >
          刷新
        </button>
        <button
          @click="pickAndUpload"
          :disabled="uploading"
          class="px-3 py-1 bg-accent/15 text-accent-text hover:bg-accent/25 rounded-full text-xs transition disabled:opacity-50"
        >
          {{ uploading ? '上传中...' : '上传歌曲' }}
        </button>
        <!-- 上传进度 -->
        <div v-if="uploading && uploadProgress < 100" class="flex items-center gap-2 text-xs text-content-3">
          <div class="w-24 h-1.5 bg-muted rounded-full overflow-hidden">
            <div class="h-full bg-accent rounded-full transition-all duration-300" :style="{ width: uploadProgress + '%' }"></div>
          </div>
          <span>{{ uploadProgress.toFixed(0) }}%</span>
        </div>
      </template>
    </PageHeader>

    <!-- 存储空间 -->
    <div v-if="cloudSize > 0" class="mb-6 p-4 bg-subtle rounded-xl">
      <div class="flex items-center justify-between text-xs mb-2">
        <span class="text-content-2">已使用 {{ formatFileSize(cloudSize) }} / {{ formatFileSize(cloudMaxSize) }}</span>
        <span class="text-content-3">{{ cloudUsagePercent }}%</span>
      </div>
      <div class="h-1.5 bg-muted rounded-full overflow-hidden">
        <div class="h-full bg-accent rounded-full transition-all duration-500" :style="{ width: cloudUsagePercent + '%' }"></div>
      </div>
    </div>

    <div v-if="!userStore.isLoggedIn" class="text-content-3 py-8">
      请先登录后查看云盘音乐
    </div>

    <div v-else-if="loading && !songs.length" class="space-y-1">
      <div v-for="i in 8" :key="i" class="flex items-center gap-3 px-3 py-2">
        <div class="w-12 h-12 bg-muted rounded animate-pulse flex-shrink-0"></div>
        <div class="flex-1 space-y-2">
          <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
          <div class="h-3 bg-muted rounded w-1/3 animate-pulse"></div>
        </div>
      </div>
    </div>

    <div v-else-if="loadError" class="flex flex-col items-center justify-center py-16 gap-3">
      <p class="text-content-2 text-sm">加载失败</p>
      <button @click="refresh" class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">重试</button>
    </div>

    <div v-else-if="!songs.length" class="text-content-3 py-8">云盘中暂无音乐</div>

    <div v-else class="space-y-1">
      <SongListItem
        v-for="(song, index) in songs"
        :key="song.id"
        :song="song"
        :index="index"
        :is-current="player.currentSong?.id === song.id"
        show-index
        show-like
        show-download
        show-duration
        show-playing-overlay
        :container-class="player.currentSong?.id === song.id ? 'bg-accent-dim hover:bg-accent-dim' : 'hover:bg-subtle'"
        @click="player.playFromList(songs, index)"
      >
        <template #actions>
          <span class="text-xs text-content-3 flex-shrink-0">{{ formatFileSize(cloudData[index]?.fileSize || 0) }}</span>
          <div class="relative flex-shrink-0">
            <button
              @click.stop="toggleMenu(song.id)"
              class="text-content-3 hover:text-content transition p-1 rounded hover:bg-muted"
              title="更多"
            >
              <IconEllipsis class="w-4 h-4 fill-current" />
            </button>
            <Transition name="fade">
              <div v-if="openMenuId === song.id" class="absolute right-0 top-full mt-1 w-40 bg-surface border border-line rounded-xl shadow-2xl overflow-hidden z-50" @click.stop>
                <button @click="showDetail(index)" class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-content-2 hover:bg-subtle transition">
                  <IconInfo style="font-size: 14px" />
                  查看详情
                </button>
                <button @click="confirmDelete(song)" class="w-full flex items-center gap-2.5 px-4 py-2.5 text-sm text-danger/80 hover:bg-danger/10 transition">
                  <IconTrash2 style="font-size: 14px" />
                  从云盘删除
                </button>
              </div>
            </Transition>
          </div>
        </template>
      </SongListItem>
    </div>

    <!-- 加载更多 -->
    <div v-if="hasMore && songs.length" class="flex justify-center py-6">
      <button
        @click="loadMore"
        :disabled="loadingMore"
        class="px-6 py-2 bg-subtle hover:bg-muted rounded-full text-sm transition disabled:opacity-50"
      >
        {{ loadingMore ? '加载中...' : '加载更多' }}
      </button>
    </div>

    <!-- 详情弹窗 -->
    <Transition name="fade">
      <div v-if="showDetailModal && detailData" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showDetailModal = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[380px] p-6 select-auto">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold truncate pr-4">{{ detailData.songName }}</h2>
            <button @click="showDetailModal = false" class="text-content-3 hover:text-content transition flex-shrink-0">
              <IconX class="w-5 h-5" />
            </button>
          </div>
          <div class="space-y-3 text-sm">
            <div class="flex justify-between">
              <span class="text-content-3">文件名</span>
              <span class="text-content-2 text-right max-w-[220px] truncate" :title="detailData.fileName">{{ detailData.fileName }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-content-3">歌手</span>
              <span class="text-content-2">{{ detailData.artist }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-content-3">专辑</span>
              <span class="text-content-2">{{ detailData.album }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-content-3">文件大小</span>
              <span class="text-content-2">{{ formatFileSize(detailData.fileSize) }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-content-3">比特率</span>
              <span class="text-content-2">{{ detailData.bitrate ? (detailData.bitrate / 1000) + ' kbps' : '未知' }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-content-3">上传时间</span>
              <span class="text-content-2">{{ detailData.addTime }}</span>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 删除确认 -->
    <Transition name="fade">
      <div v-if="showDeleteConfirm" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showDeleteConfirm = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-6 select-auto">
          <h2 class="text-lg font-semibold text-content mb-1">确认删除</h2>
          <p class="text-sm text-content-2 mb-5">确定要从云盘删除「{{ deleteTarget?.name }}」吗？</p>
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
import { ref, computed, onMounted, onActivated, onBeforeUnmount } from 'vue';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { showToast } from '../composables/useToast';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate } from '../composables/usePageCache';
import { formatFileSize } from '../composables/useLocalMusic';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import SongListItem from '../components/SongListItem.vue';
import PageHeader from '../components/PageHeader.vue';
import IconEllipsis from '~icons/lucide/ellipsis';
import IconInfo from '~icons/lucide/info';
import IconTrash2 from '~icons/lucide/trash-2';
import IconX from '~icons/lucide/x';

defineOptions({ name: 'CloudMusicView' });

const player = usePlayerStore();
const userStore = useUserStore();

interface CloudItem {
  songId: number;
  fileSize: number;
  fileName: string;
  bitrate: number;
  addTime: string;
  artist: string;
  album: string;
  songName: string;
}

const songs = ref<Song[]>([]);
const cloudData = ref<CloudItem[]>([]);
const loading = ref(true);
const loadingMore = ref(false);
const loadError = ref(false);
const hasMore = ref(false);
const totalCount = ref(0);
const openMenuId = ref<number | null>(null);
const showDeleteConfirm = ref(false);
const showDetailModal = ref(false);
const detailData = ref<CloudItem | null>(null);
const deleteTarget = ref<Song | null>(null);
const cloudSize = ref(0);
const cloudMaxSize = ref(0);
const uploading = ref(false);
const uploadProgress = ref(0);
let unlistenProgress: UnlistenFn | null = null;

const cloudUsagePercent = computed(() => {
  if (cloudMaxSize.value === 0) return 0;
  return Math.min(100, Math.round(cloudSize.value / cloudMaxSize.value * 100));
});

const LIMIT = 30;
let currentOffset = 0;

function formatTimestamp(ts: number): string {
  if (!ts) return '未知';
  return new Date(ts).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
}

function toggleMenu(id: number) {
  openMenuId.value = openMenuId.value === id ? null : id;
}

function closeMenu() {
  openMenuId.value = null;
}

onMounted(() => {
  document.addEventListener('click', closeMenu);
  // 监听上传进度事件
  listen<{ filename: string; progress: number; uploaded: number; total: number }>('cloud-upload-progress', (e) => {
    uploadProgress.value = e.payload.progress;
  }).then(fn => { unlistenProgress = fn; });
});
onBeforeUnmount(() => {
  document.removeEventListener('click', closeMenu);
  unlistenProgress?.();
});

async function fetchCloud(offset = 0, append = false) {
  if (!userStore.isLoggedIn) {
    loading.value = false;
    return;
  }

  if (!append) {
    loading.value = true;
    loadError.value = false;
  } else {
    loadingMore.value = true;
  }

  try {
    const jsonStr = await MusicApi.userCloud(LIMIT, offset);
    const data = JSON.parse(jsonStr);
    const items = data.data || [];

    const newSongs = items.map((item: any) => {
      const s = item.simpleSong || {};
      return normalizeSong({
        ...s,
        id: s.id || item.songId,
        name: s.name || item.fileName,
        ar: s.ar || (item.artist ? [{ name: item.artist }] : []),
        al: s.al || { name: item.album || '未知专辑' },
        dt: s.dt || item.duration,
      });
    });

    const newCloudData: CloudItem[] = items.map((item: any) => ({
      songId: item.songId,
      fileSize: item.fileSize || 0,
      fileName: item.fileName || '',
      bitrate: item.bitrate || 0,
      addTime: formatTimestamp(item.addTime),
      artist: item.artist || (item.simpleSong?.ar || []).map((a: any) => a.name).join(' / ') || '未知歌手',
      album: item.album || item.simpleSong?.al?.name || '未知专辑',
      songName: item.simpleSong?.name || item.fileName?.replace(/\.\w+$/, '') || '未知歌曲',
    }));

    if (append) {
      songs.value = [...songs.value, ...newSongs];
      cloudData.value = [...cloudData.value, ...newCloudData];
    } else {
      songs.value = newSongs;
      cloudData.value = newCloudData;
    }

    totalCount.value = data.count || songs.value.length;
    currentOffset = offset + items.length;
    hasMore.value = songs.value.length < totalCount.value;
    cloudSize.value = data.size || 0;
    cloudMaxSize.value = data.maxSize || 0;

    if (!append) {
      pageCacheSet('cloudMusic', {
        songs: songs.value, cloudData: cloudData.value, totalCount: totalCount.value,
        hasMore: hasMore.value, offset: currentOffset,
        cloudSize: cloudSize.value, cloudMaxSize: cloudMaxSize.value,
      });
    }
  } catch (e) {
    console.error('获取云盘音乐失败', e);
    if (!append) loadError.value = true;
    else showToast('加载更多失败', 'error');
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

function refresh() {
  pageCacheInvalidate('cloudMusic');
  currentOffset = 0;
  fetchCloud(0, false);
}

function loadMore() {
  fetchCloud(currentOffset, true);
}

async function pickAndUpload() {
  const selected = await open({
    multiple: true,
    filters: [{ name: '音频文件', extensions: ['mp3', 'flac', 'wav', 'ogg', 'aac', 'm4a'] }],
    title: '选择要上传的歌曲',
  });
  if (!selected) return;

  const paths = Array.isArray(selected) ? selected : [selected];
  uploading.value = true;
  uploadProgress.value = 0;

  for (const filePath of paths) {
    uploadProgress.value = 0;
    try {
      await MusicApi.cloudUpload(filePath);
      showToast('上传成功', 'success');
    } catch (e: any) {
      showToast(`上传失败: ${e || '未知错误'}`, 'error');
    }
  }

  uploading.value = false;
  uploadProgress.value = 0;
  // 等待服务端完全提交后再刷新列表
  setTimeout(() => refresh(), 1000);
}

function showDetail(index: number) {
  openMenuId.value = null;
  detailData.value = cloudData.value[index] || null;
  showDetailModal.value = true;
}

function confirmDelete(song: Song) {
  openMenuId.value = null;
  deleteTarget.value = song;
  showDeleteConfirm.value = true;
}

async function doDelete() {
  if (!deleteTarget.value) return;
  try {
    await MusicApi.userCloudDel(deleteTarget.value.id);
    const targetId = deleteTarget.value.id;
    const idx = songs.value.findIndex(s => s.id === targetId);
    songs.value = songs.value.filter(s => s.id !== targetId);
    if (idx !== -1) cloudData.value.splice(idx, 1);
    totalCount.value = Math.max(0, totalCount.value - 1);
    pageCacheInvalidate('cloudMusic');
    showToast('已从云盘删除', 'success');
  } catch {
    showToast('删除失败', 'error');
  }
  showDeleteConfirm.value = false;
  deleteTarget.value = null;
}

onMounted(() => {
  if (!userStore.isLoggedIn) {
    loading.value = false;
    return;
  }
  const cached = pageCacheGet('cloudMusic');
  if (cached) {
    songs.value = cached.songs;
    cloudData.value = cached.cloudData;
    totalCount.value = cached.totalCount;
    hasMore.value = cached.hasMore;
    currentOffset = cached.offset;
    cloudSize.value = cached.cloudSize || 0;
    cloudMaxSize.value = cached.cloudMaxSize || 0;
    loading.value = false;
    return;
  }
  fetchCloud();
});

onActivated(() => {
  if (loadError.value) refresh();
});
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
