<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">我喜欢的音乐</h1>
      <button
        v-if="songs.length"
        @click="player.playAll(songs)"
        class="px-5 py-2 bg-accent hover:bg-accent-hover rounded-full text-white font-medium transition flex items-center gap-2"
      >
        <IconPlay class="w-4 h-4 fill-current" />
        播放全部
      </button>
    </PageHeader>
    <div v-if="!userStore.isLoggedIn" class="text-content-2">
      请先登录后查看喜欢的音乐
    </div>
    <div v-else-if="loading" class="space-y-1">
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
    <div v-else-if="songs.length === 0" class="text-content-2">暂无喜欢的音乐</div>
    <VirtualSongList
      v-else
      :songs="songs"
      :current-song-id="player.currentSong?.id"
      show-index
      show-like
      show-download
      show-menu
      show-duration
      show-playing-overlay
      @song-click="(_s, i) => player.playFromList(songs, i)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onActivated, watch } from 'vue';
import { MusicApi } from '../api';
import VirtualSongList from '../components/VirtualSongList.vue';
import PageHeader from '../components/PageHeader.vue';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import IconPlay from '~icons/lucide/play';

defineOptions({ name: 'FavoriteSongsView' });

const player = usePlayerStore();
const userStore = useUserStore();
const songs = ref<Song[]>([]);
const loading = ref(true);
const loadError = ref(false);

async function loadData(force = false) {
  if (!userStore.isLoggedIn) {
    loading.value = false;
    return;
  }
  if (!force) {
    const cached = pageCacheGet('favoriteSongs');
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
    const playlistJson: string = await MusicApi.userPlaylist(userStore.user!.userId);
    const playlistData = JSON.parse(playlistJson);
    const created = (playlistData.playlist || []).filter((p: any) => !p.subscribed);
    if (created.length === 0) {
      loading.value = false;
      return;
    }
    const likePlaylistId = created[0].id;
    const trackJson: string = await MusicApi.playlistTrackAll(likePlaylistId);
    const trackData = JSON.parse(trackJson);
    songs.value = (trackData.songs || []).map(normalizeSong);
    pageCacheSet('favoriteSongs', songs.value);
  } catch (e) {
    console.error('加载我喜欢的音乐失败', e);
    loadError.value = true;
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);

onActivated(() => {
  if (loadError.value) {
    loadData(true);
  } else if (pageCacheIsStale('favoriteSongs')) {
    loadData();
  }
});

watch(() => navigator.onLine, (val, old) => {
  if (val && !old && userStore.isLoggedIn && songs.value.length === 0) {
    pageCacheInvalidate('favoriteSongs');
    loadData();
  }
});
</script>
