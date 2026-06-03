<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">本地音乐</h1>
      <span v-if="songs.length" class="text-xs text-content-3">{{ songs.length }} 首</span>
      <template #actions>
        <button
          @click="refresh"
          class="px-3 py-1 bg-muted hover:bg-emphasis rounded-full text-xs transition"
        >
          刷新
        </button>
        <button
          @click="showFolderModal = true"
          class="text-content-3 hover:text-content transition p-1 rounded hover:bg-muted"
          title="文件夹管理"
        >
          <IconEllipsis class="w-5 h-5 fill-current" />
        </button>
      </template>
    </PageHeader>

    <div v-if="loading" class="space-y-1">
      <div v-for="i in 6" :key="i" class="flex items-center gap-3 px-3 py-2">
        <div class="w-12 h-12 bg-muted rounded animate-pulse flex-shrink-0"></div>
        <div class="flex-1 space-y-2">
          <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
          <div class="h-3 bg-muted rounded w-1/3 animate-pulse"></div>
        </div>
      </div>
    </div>
    <div v-else-if="settings.localMusicPaths.length === 0" class="text-content-3 py-4">
      请先添加要扫描的文件夹
    </div>
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
        </template>
      </SongListItem>
    </div>

    <!-- 文件夹管理弹窗 -->
    <Transition name="fade">
      <div v-if="showFolderModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showFolderModal = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[420px] p-6 select-auto">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold">扫描文件夹</h2>
            <button @click="showFolderModal = false" class="text-content-3 hover:text-content transition">
              <IconX class="w-5 h-5" />
            </button>
          </div>
          <div v-if="settings.localMusicPaths.length === 0" class="text-sm text-content-3 py-4 text-center">
            未添加任何文件夹
          </div>
          <div v-else class="space-y-1.5 max-h-60 overflow-y-auto mb-4">
            <div
              v-for="p in settings.localMusicPaths"
              :key="p"
              class="flex items-center gap-2 px-3 py-2 bg-subtle rounded-lg group"
            >
              <IconFolder class="w-4 h-4 text-content-3 flex-shrink-0" />
              <span class="text-sm text-content-2 truncate flex-1" :title="p">{{ p }}</span>
              <button
                @click="settings.removeLocalMusicPath(p)"
                class="text-content-4 hover:text-danger transition opacity-0 group-hover:opacity-100"
                title="移除"
              >
                <IconX class="w-4 h-4" />
              </button>
            </div>
          </div>
          <button
            @click="addFolder"
            class="w-full py-2.5 rounded-lg bg-accent/15 text-accent-text hover:bg-accent/25 text-sm font-medium transition"
          >
            添加文件夹
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, watch } from 'vue';
import { DownloadApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { useSettingsStore } from '../stores/settings';
import { pageCacheSet, pageCacheIsStale } from '../composables/usePageCache';
import { formatFileSize, localSongToSong, fetchMissingCovers, type LocalSong } from '../composables/useLocalMusic';
import { open } from '@tauri-apps/plugin-dialog';
import SongListItem from '../components/SongListItem.vue';
import PageHeader from '../components/PageHeader.vue';
import IconEllipsis from '~icons/lucide/ellipsis';
import IconFolder from '~icons/lucide/folder';
import IconX from '~icons/lucide/x';

defineOptions({ name: 'LocalMusicView' });

const player = usePlayerStore();
const settings = useSettingsStore();

const songs = ref<LocalSong[]>([]);
const loading = ref(true);
const showFolderModal = ref(false);

const normalizedSongs = computed(() => songs.value.map(localSongToSong));

async function addFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择音乐文件夹',
  });
  if (selected) {
    settings.addLocalMusicPath(selected);
  }
}

async function refresh() {
  if (settings.localMusicPaths.length === 0) {
    songs.value = [];
    loading.value = false;
    return;
  }
  loading.value = true;
  try {
    const list = await DownloadApi.scanLocalFolders(settings.localMusicPaths);
    songs.value = list;
    pageCacheSet('localMusic', list);
    fetchMissingCovers(songs.value);
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

onMounted(refresh);

onActivated(() => {
  if (pageCacheIsStale('localMusic')) refresh();
});

watch(() => settings.localMusicPaths, () => { refresh(); }, { deep: true });
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
