<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <div class="flex items-center gap-4 mb-6">
      <h1 class="text-2xl font-bold">我喜欢的音乐</h1>
      <button
        v-if="songs.length"
        @click="player.playAll(songs)"
        class="px-4 py-1.5 bg-muted hover:bg-emphasis rounded-full text-sm transition"
      >
        播放全部
      </button>
    </div>
    <div v-if="!userStore.isLoggedIn" class="text-content-2">
      请先登录后查看喜欢的音乐
    </div>
    <div v-else-if="loading" class="text-content-2">加载中...</div>
    <div v-else-if="songs.length === 0" class="text-content-2">暂无喜欢的音乐</div>
    <div v-else class="space-y-2">
      <div
        v-for="(song, index) in songs"
        :key="song.id"
        @click="player.play(song)"
        class="flex items-center gap-4 p-3 rounded-xl hover:bg-subtle transition cursor-pointer"
      >
        <span class="text-xs text-content-3 w-6 text-right">{{ index + 1 }}</span>
        <img :src="song.al?.picUrl" class="w-10 h-10 rounded object-cover" />
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">{{ song.name }}</p>
          <p class="text-xs text-content-2 truncate">
            {{ song.ar?.map((a: any) => a.name).join(' / ') }}
          </p>
        </div>
        <button @click.stop="player.toggleLike(song.id)" class="text-content-3 hover:text-danger transition flex-shrink-0">
          <svg v-if="player.isLiked(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="text-danger"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
        </button>
        <span class="text-xs text-content-3">{{ formatDuration(song.dt) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { normalizeSong } from '../utils/song';
import { formatDuration } from '../utils/format';

const player = usePlayerStore();
const userStore = useUserStore();
const songs = ref<any[]>([]);
const loading = ref(true);

onMounted(async () => {
  if (!userStore.isLoggedIn) {
    loading.value = false;
    return;
  }
  try {
    const playlistJson: string = await invoke('user_playlist', { uid: userStore.user!.userId });
    const playlistData = JSON.parse(playlistJson);
    const created = (playlistData.playlist || []).filter((p: any) => !p.subscribed);
    if (created.length === 0) {
      loading.value = false;
      return;
    }
    const likePlaylistId = created[0].id;
    const trackJson: string = await invoke('playlist_track_all', { query: { id: likePlaylistId } });
    const trackData = JSON.parse(trackJson);
    songs.value = (trackData.songs || []).map(normalizeSong);
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
});
</script>
