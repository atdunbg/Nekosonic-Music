<template>
  <div class="p-8 text-white">
    <button @click="$router.back()" class="mb-4 text-gray-400 hover:text-white transition">
      ← 返回
    </button>

    <!-- 歌单信息 -->
    <div v-if="playlist" class="flex gap-6 mb-8">
      <img :src="playlist.coverImgUrl" class="w-40 h-40 rounded-xl object-cover shadow-lg" />
      <div>
        <h1 class="text-2xl font-bold">{{ playlist.name }}</h1>
        <p class="text-sm text-gray-400 mt-2">{{ playlist.description }}</p>
        <p class="text-xs text-gray-500 mt-2">
          {{ playlist.trackCount }} 首歌曲 · 播放 {{ playlist.playCount }} 次
        </p>
        <button
          @click="playAll"
          class="mt-4 px-4 py-2 bg-green-500 hover:bg-green-600 rounded-full text-white font-medium transition"
        >
          播放全部
        </button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-gray-400">加载中...</div>

    <!-- 歌曲列表 -->
    <div v-else class="space-y-2">
      <div
        v-for="(song, index) in songs"
        :key="song.id"
        @click="playSingle(song)"
        class="flex items-center gap-4 p-3 rounded-xl hover:bg-white/5 transition cursor-pointer"
      >
        <span class="text-xs text-gray-500 w-6 text-right">{{ index + 1 }}</span>
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
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';

const route = useRoute();
const player = usePlayerStore();

const playlist = ref<any>(null);
const songs = ref<any[]>([]);
const loading = ref(true);

onMounted(async () => {
  const id = Number(route.params.id);
  try {
    const jsonStr: string = await invoke('get_playlist_detail', { id });
    const data = JSON.parse(jsonStr);
    playlist.value = data.playlist;
    songs.value = data.playlist.tracks || [];
  } catch (e) {
    console.error('获取歌单详情失败', e);
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

async function playSingle(song: any) {
  player.play(song);
}

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}
</script>