<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">本地音乐</h1>
      <span v-if="songs.length" class="text-xs text-content-3">{{ songs.length }} 首</span>
      <template #actions>
        <button @click="cycleSort" class="px-3 py-1 bg-muted hover:bg-emphasis rounded-full text-xs transition-all flex items-center justify-center gap-1 whitespace-nowrap">
          <IconArrowUpDown class="w-3 h-3" />
          {{ sortLabel }}
        </button>
        <button @click="refresh" class="px-3 py-1 bg-muted hover:bg-emphasis rounded-full text-xs transition">刷新</button>
        <button @click="showFolderModal = true" class="px-3 py-1 bg-muted hover:bg-emphasis rounded-full text-xs transition">扫描目录</button>
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
    <div v-else-if="settings.localMusicFolders.length === 0" class="text-content-3 py-4">
      请先添加要扫描的文件夹
    </div>
    <div v-else-if="settings.enabledMusicPaths.length === 0" class="text-content-3 py-4">
      请至少启用一个扫描文件夹
    </div>
    <div v-else-if="songs.length === 0" class="text-content-3">
      当前文件夹下没有音乐文件，支持 mp3、flac、wav、ogg、aac、m4a 格式
    </div>
    <div v-else class="space-y-2">
      <SongListItem
        v-for="(song, index) in sortedSongs"
        :key="song.id + '-' + index"
        :song="sortedNormalized[index]"
        :index="index"
        :is-current="player.currentSong?.id === song.id"
        show-index
        show-duration
        show-playing-overlay
        :container-class="player.currentSong?.id === song.id ? 'bg-accent-dim hover:bg-accent-dim' : 'hover:bg-subtle'"
        @click="player.playFromList(sortedNormalized, index)"
      >
        <template #actions>
          <span class="text-xs text-content-3 flex-shrink-0">{{ formatFileSize(song.fileSize) }}</span>
          <div class="relative flex-shrink-0" :ref="(el: any) => menuRefs[song.id] = el">
            <button @click.stop="toggleMenu(song.id)" class="text-content-3 hover:text-content transition p-1 rounded-md hover:bg-subtle">
              <IconEllipsis class="w-4 h-4 fill-current" />
            </button>
            <div v-if="openMenuId === song.id"
              class="absolute right-0 top-full mt-1 bg-surface border border-line rounded-xl shadow-xl z-50 py-1 min-w-[140px]">
              <button @click.stop="openFolder(song.path)" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-content-2 hover:bg-subtle hover:text-content transition whitespace-nowrap">
                <IconFolderOpen class="w-3.5 h-3.5" />
                打开所在文件夹
              </button>
            </div>
          </div>
        </template>
      </SongListItem>
    </div>

    <!-- 文件夹管理弹窗 -->
    <Transition name="fade">
      <div v-if="showFolderModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showFolderModal = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[420px] p-6 select-auto">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold">扫描目录</h2>
            <button @click="showFolderModal = false" class="text-content-3 hover:text-content transition">
              <IconX class="w-5 h-5" />
            </button>
          </div>
          <div v-if="settings.localMusicFolders.length === 0" class="text-sm text-content-3 py-4 text-center">
            未添加任何文件夹
          </div>
          <div v-else class="space-y-1.5 max-h-60 overflow-y-auto mb-4">
            <div v-for="folder in settings.localMusicFolders" :key="folder.path" class="flex items-center gap-2 px-3 py-2 bg-subtle rounded-lg group">
              <button @click="settings.toggleLocalMusicFolder(folder.path)" class="flex-shrink-0" :title="folder.enabled ? '点击禁用' : '点击启用'">
                <IconCheckSquare v-if="folder.enabled" class="w-4 h-4 text-accent-text" />
                <IconSquare v-else class="w-4 h-4 text-content-4" />
              </button>
              <IconFolder class="w-4 h-4 text-content-3 flex-shrink-0" />
              <span class="text-sm truncate flex-1" :class="folder.enabled ? 'text-content-2' : 'text-content-4 line-through'" :title="folder.path">{{ folder.path }}</span>
              <button @click="settings.removeLocalMusicPath(folder.path)" class="text-content-4 hover:text-danger transition opacity-0 group-hover:opacity-100" title="移除">
                <IconX class="w-4 h-4" />
              </button>
            </div>
          </div>
          <button @click="addFolder" class="w-full py-2.5 rounded-lg bg-accent/15 text-accent-text hover:bg-accent/25 text-sm font-medium transition">
            添加文件夹
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, watch, onBeforeUnmount } from 'vue';
import { AppApi, DownloadApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { useSettingsStore } from '../stores/settings';
import { pageCacheSet, pageCacheIsStale } from '../composables/usePageCache';
import { formatFileSize, localSongToSong, fetchMissingCovers, type LocalSong } from '../composables/useLocalMusic';
import { showToast } from '../composables/useToast';
import { open } from '@tauri-apps/plugin-dialog';
import SongListItem from '../components/SongListItem.vue';
import PageHeader from '../components/PageHeader.vue';
import IconFolder from '~icons/lucide/folder';
import IconFolderOpen from '~icons/lucide/folder-open';
import IconX from '~icons/lucide/x';
import IconArrowUpDown from '~icons/lucide/arrow-up-down';
import IconCheckSquare from '~icons/lucide/check-square';
import IconSquare from '~icons/lucide/square';
import IconEllipsis from '~icons/lucide/ellipsis';

defineOptions({ name: 'LocalMusicView' });

const player = usePlayerStore();
const settings = useSettingsStore();

const songs = ref<LocalSong[]>([]);
const loading = ref(true);
const showFolderModal = ref(false);

// 排序：点击循环切换
type SortKey = 'default' | 'name' | 'size';
const SORT_CYCLE: SortKey[] = ['default', 'name', 'size'];
const SORT_LABELS: Record<SortKey, string> = { default: '默认', name: '名称', size: '大小' };
const sortBy = ref<SortKey>('default');

const sortLabel = computed(() => SORT_LABELS[sortBy.value]);

function cycleSort() {
  const idx = SORT_CYCLE.indexOf(sortBy.value);
  sortBy.value = SORT_CYCLE[(idx + 1) % SORT_CYCLE.length];
}

const sortedSongs = computed(() => {
  const list = [...songs.value];
  if (sortBy.value === 'name') {
    list.sort((a, b) => a.name.localeCompare(b.name, 'zh-Hans-CN'));
  } else if (sortBy.value === 'size') {
    list.sort((a, b) => b.fileSize - a.fileSize);
  }
  return list;
});

const sortedNormalized = computed(() => sortedSongs.value.map(localSongToSong));

// 三点菜单
const openMenuId = ref<number | null>(null);
const menuRefs: Record<number, HTMLElement | null> = {};

function toggleMenu(id: number) {
  openMenuId.value = openMenuId.value === id ? null : id;
}

async function openFolder(path: string) {
  openMenuId.value = null;
  try {
    await AppApi.showItemInFolder(path);
  } catch (e: any) {
    showToast(e.toString(), 'error');
  }
}

function onClickOutside(e: MouseEvent) {
  if (openMenuId.value !== null) {
    const el = menuRefs[openMenuId.value];
    if (el && !el.contains(e.target as Node)) {
      openMenuId.value = null;
    }
  }
}

onMounted(() => document.addEventListener('click', onClickOutside));
onBeforeUnmount(() => document.removeEventListener('click', onClickOutside));

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
  const paths = settings.enabledMusicPaths;
  if (paths.length === 0) {
    songs.value = [];
    loading.value = false;
    return;
  }
  loading.value = true;
  try {
    const list = await DownloadApi.scanLocalFolders(paths);
    songs.value = list;
    pageCacheSet('localMusic', list);
    fetchMissingCovers(songs.value);
  } catch (e) {
    console.error('扫描本地音乐文件夹失败', e);
  } finally {
    loading.value = false;
  }
}

onMounted(refresh);

onActivated(() => {
  if (pageCacheIsStale('localMusic')) refresh();
});

watch(() => settings.enabledMusicPaths, () => { refresh(); }, { deep: true });
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
