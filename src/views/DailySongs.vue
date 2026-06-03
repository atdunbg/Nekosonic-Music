<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">每日推荐</h1>
      <button
        v-if="songs.length > 0"
        @click="player.playAll(songs)"
        class="px-4 py-2 bg-accent hover:bg-accent-hover rounded-full text-white font-medium transition text-sm"
      >
        播放全部
      </button>
    </PageHeader>
    <div v-if="loading" class="space-y-1">
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
      <button @click="loadData(true)" class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">重试</button>
    </div>
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
import { MusicApi } from '../api';
import SongListItem from '../components/SongListItem.vue';
import PageHeader from '../components/PageHeader.vue';
import { usePlayerStore } from '../stores/player';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { normalizeSong, type Song } from '../utils/song';

defineOptions({ name: 'DailySongsView' });

const player = usePlayerStore();
const songs = ref<Song[]>([]);
const loading = ref(true);
const loadError = ref(false);

function isCurrentSong(songId: number): boolean {
  return player.currentSong?.id === songId;
}

async function loadData(force = false) {
  if (!force) {
    const cached = pageCacheGet('dailySongs');
    if (cached) {
      songs.value = cached;
      loading.value = false;
      loadError.value = false;
      return;
    }
  }
  loading.value = true;
  try {
    loadError.value = false;
    const jsonStr: string = await MusicApi.recommendSongs();
    const data = JSON.parse(jsonStr);
    songs.value = (data.data?.dailySongs || []).map(normalizeSong);
    pageCacheSet('dailySongs', songs.value);
  } catch (e) {
    console.error(e);
    loadError.value = true;
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);

onActivated(() => {
  if (loadError.value) {
    loadData(true);
  } else if (pageCacheIsStale('dailySongs')) {
    loadData();
  }
});

watch(() => navigator.onLine, (val, old) => {
  if (val && !old && songs.value.length === 0) {
    pageCacheInvalidate('dailySongs');
    loadData();
  }
});
</script>
