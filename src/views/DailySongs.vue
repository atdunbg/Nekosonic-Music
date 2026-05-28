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
      />
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
