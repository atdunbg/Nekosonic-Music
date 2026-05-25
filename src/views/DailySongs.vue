<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">每日推荐</h1>
      <button
        v-if="songs.length > 0"
        @click="player.playAll(songs)"
        class="px-4 py-2 bg-accent hover:bg-accent-hover rounded-full text-white font-medium transition text-sm"
      >
        播放全部
      </button>
    </div>
    <div v-if="loading" class="text-content-2">加载中...</div>
    <div v-else class="space-y-2">
      <SongListItem
        v-for="(song, index) in songs"
        :key="song.id"
        :song="song"
        :index="index"
        :is-current="isCurrentSong(song.id)"
        show-index
        show-like
        show-download
        show-menu
        show-duration
        show-playing-overlay
        :container-class="isCurrentSong(song.id) ? 'bg-accent-dim hover:bg-accent-dim' : 'hover:bg-subtle'"
        @click="player.playFromList(songs, index)"
      >
        <template #index="{ index: idx, isCurrent }">
          <div class="w-6 text-right flex-shrink-0 flex items-center justify-end h-5">
            <div v-if="isCurrent" class="flex items-center justify-end">
              <div class="flex items-center gap-[3px] h-4">
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 50%; animation-delay: 0ms"></span>
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 100%; animation-delay: 150ms"></span>
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 35%; animation-delay: 300ms"></span>
              </div>
            </div>
            <template v-else>
              <span class="text-xs text-content-3 group-hover:hidden">{{ idx + 1 }}</span>
              <svg class="hidden group-hover:block text-content" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg>
            </template>
          </div>
        </template>
      </SongListItem>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onActivated, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SongListItem from '../components/SongListItem.vue';
import { usePlayerStore } from '../stores/player';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { normalizeSong, type Song } from '../utils/song';
import { useOnlineStatus } from '../composables/useOnlineStatus';

defineOptions({ name: 'DailySongsView' });

const player = usePlayerStore();
const { isOnline } = useOnlineStatus();
const songs = ref<Song[]>([]);
const loading = ref(true);

function isCurrentSong(songId: number): boolean {
  return player.currentSong?.id === songId;
}

async function loadData() {
  const cached = pageCacheGet('dailySongs');
  if (cached) {
    songs.value = cached;
    loading.value = false;
    return;
  }
  loading.value = true;
  try {
    const jsonStr: string = await invoke('recommend_songs');
    const data = JSON.parse(jsonStr);
    songs.value = (data.data?.dailySongs || []).map(normalizeSong);
    pageCacheSet('dailySongs', songs.value);
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);

onActivated(() => {
  if (pageCacheIsStale('dailySongs')) loadData();
});

watch(isOnline, (val, old) => {
  if (val && !old && songs.value.length === 0) {
    pageCacheInvalidate('dailySongs');
    loadData();
  }
});
</script>
