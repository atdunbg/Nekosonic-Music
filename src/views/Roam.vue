<template>
  <div class="p-8 text-content flex flex-col items-center justify-center min-h-full">
    <div v-if="!currentSong" class="text-center">
      <p class="text-content-2 mb-4">私人漫游未启动</p>
      <button
        @click="startFm"
        class="px-6 py-2 bg-muted hover:bg-emphasis rounded-full transition"
      >
        开始漫游
      </button>
    </div>

    <template v-else>
      <img
        v-if="coverUrl && !coverError"
        :src="coverUrl"
        class="w-80 h-80 rounded-3xl object-cover shadow-2xl mb-8"
        @error="coverError = true"
      />
      <div
        v-else
        class="w-80 h-80 rounded-3xl bg-muted flex items-center justify-center shadow-2xl mb-8"
      >
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-content-3"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      </div>

      <h1 class="text-3xl font-bold mb-2">{{ currentSong.name }}</h1>
      <p class="text-lg text-content-2 mb-8">
        {{ artists }}
      </p>

      <div class="flex items-center gap-8">
        <button
          @click="player.toggle()"
          class="w-16 h-16 flex items-center justify-center rounded-full bg-muted hover:bg-emphasis transition border border-emphasis"
        >
          <svg v-if="player.playing" width="28" height="28" viewBox="0 0 16 16" fill="currentColor">
            <rect x="3" y="2" width="3" height="12" rx="0.5" />
            <rect x="10" y="2" width="3" height="12" rx="0.5" />
          </svg>
          <svg v-else width="28" height="28" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4 2.5v11l9-5.5z" />
          </svg>
        </button>
        <button
          @click="nextSong"
          class="text-content-2 hover:text-content transition"
        >
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19"/></svg>
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { usePlayerStore } from '../stores/player';
import { invoke } from '@tauri-apps/api/core';
import { normalizeSong } from '../utils/song';

const player = usePlayerStore();
const coverError = ref(false);

const currentSong = computed(() => {
  if (player.isFmMode && player.currentSong) {
    return player.currentSong;
  }
  return null;
});

const coverUrl = computed(() => {
  if (!currentSong.value) return '';
  return currentSong.value.al?.picUrl || currentSong.value.album?.picUrl || '';
});

watch(coverUrl, () => { coverError.value = false; });

const artists = computed(() => {
  if (!currentSong.value) return '';
  return currentSong.value.ar?.map((a: any) => a.name).join(' / ') ||
         currentSong.value.artists?.map((a: any) => a.name).join(' / ') || '';
});

onMounted(async () => {
  if (!player.isFmMode || !player.currentSong) {
    await startFm();
  }
});

async function startFm() {
  try {
    const jsonStr: string = await invoke('personal_fm');
    const data = JSON.parse(jsonStr);
    const songs = data.data || data;
    if (songs && songs.length > 0) {
      const song = normalizeSong(songs[0]);
      player.enableFmMode(nextSong);
      await player.playFmSong(song);
    }
  } catch (e) {
    console.error('启动漫游失败', e);
  }
}

async function nextSong() {
  await startFm();
}
</script>
