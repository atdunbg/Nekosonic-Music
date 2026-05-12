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
      <div
        v-for="(song, index) in songs"
        :key="song.id"
        @click="player.play(song)"
        class="flex items-center gap-4 p-3 rounded-xl hover:bg-subtle transition cursor-pointer group"
        :class="{ 'bg-accent-dim': isCurrentSong(song.id) }"
      >
        <div class="w-6 text-right flex-shrink-0 flex items-center justify-end h-5">
          <div v-if="isCurrentSong(song.id)" class="flex items-center justify-end">
            <div class="flex items-center gap-[3px] h-4">
              <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 50%; animation-delay: 0ms"></span>
              <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 100%; animation-delay: 150ms"></span>
              <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 35%; animation-delay: 300ms"></span>
            </div>
          </div>
          <template v-else>
            <span class="text-xs text-content-3 group-hover:hidden">{{ index + 1 }}</span>
            <svg class="hidden group-hover:block text-content" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg>
          </template>
        </div>
        <img :src="song.al?.picUrl" class="w-10 h-10 rounded object-cover flex-shrink-0" />
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate" :class="isCurrentSong(song.id) ? 'text-accent-text' : ''">{{ song.name }}</p>
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
import { formatDuration } from '../utils/format';

const player = usePlayerStore();
const songs = ref<any[]>([]);
const loading = ref(true);

function isCurrentSong(songId: number): boolean {
  return player.currentSong?.id === songId;
}

onMounted(async () => {
  try {
    const jsonStr: string = await invoke('recommend_songs');
    const data = JSON.parse(jsonStr);
    songs.value = data.data?.dailySongs || [];
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
});
</script>
