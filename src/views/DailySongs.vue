<template>
  <div class="p-8 text-white">
    <button @click="$router.back()" class="mb-4 text-gray-400 hover:text-white transition">
      ← 返回
    </button>
    <h1 class="text-2xl font-bold mb-6">每日推荐</h1>
    <div v-if="loading" class="text-gray-400">加载中...</div>
    <div v-else class="space-y-2">
      <div
        v-for="(song, index) in songs"
        :key="song.id"
        @click="player.play(song)"
        class="flex items-center gap-4 p-3 rounded-xl hover:bg-white/5 transition cursor-pointer"
      >
        <span class="text-xs text-gray-500 w-6 text-right">{{ index + 1 }}</span>
        <img :src="song.al?.picUrl" class="w-10 h-10 rounded object-cover" />
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">{{ song.name }}</p>
          <p class="text-xs text-gray-400 truncate">
            {{ song.ar?.map((a: any) => a.name).join(' / ') }}
          </p>
        </div>
        <span class="text-xs text-gray-500">{{ formatDuration(song.dt) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';

const player = usePlayerStore();
const songs = ref<any[]>([]);
const loading = ref(true);

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

function formatDuration(ms: number): string {
  const sec = Math.floor(ms / 1000);
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}
</script>