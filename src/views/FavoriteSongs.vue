<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">我喜欢的音乐</h1>
      <button
        v-if="songs.length"
        @click="player.playAll(songs)"
        class="px-5 py-2 bg-accent hover:bg-accent-hover rounded-full text-white font-medium transition flex items-center gap-2"
      >
        <IconPlay class="w-4 h-4 fill-current" />
        播放全部
      </button>
    </div>
    <div v-if="!userStore.isLoggedIn" class="text-content-2">
      请先登录后查看喜欢的音乐
    </div>
    <div v-else-if="loading" class="text-content-2">加载中...</div>
    <div v-else-if="songs.length === 0" class="text-content-2">暂无喜欢的音乐</div>
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
        show-menu
        show-duration
        show-playing-overlay
        :container-class="player.currentSong?.id === song.id ? 'bg-accent-dim hover:bg-accent-dim' : 'hover:bg-subtle'"
        @click="player.playFromList(songs, index)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onActivated, watch } from 'vue';
import { MusicApi } from '../api';
import SongListItem from '../components/SongListItem.vue';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { useOnlineStatus } from '../composables/useOnlineStatus';
import IconPlay from '~icons/lucide/play';

defineOptions({ name: 'FavoriteSongsView' });

const player = usePlayerStore();
const userStore = useUserStore();
const { isOnline } = useOnlineStatus();
const songs = ref<Song[]>([]);
const loading = ref(true);

async function loadData() {
  if (!userStore.isLoggedIn) {
    loading.value = false;
    return;
  }
  const cached = pageCacheGet('favoriteSongs');
  if (cached) {
    songs.value = cached;
    loading.value = false;
    return;
  }
  loading.value = true;
  try {
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
    console.error(e);
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);

onActivated(() => {
  if (pageCacheIsStale('favoriteSongs')) loadData();
});

watch(isOnline, (val, old) => {
  if (val && !old && userStore.isLoggedIn && songs.value.length === 0) {
    pageCacheInvalidate('favoriteSongs');
    loadData();
  }
});
</script>
