<template>
  <div class="p-8 text-white flex flex-col items-center justify-center min-h-full">
    <!-- 无歌曲时提示 -->
    <div v-if="!currentSong" class="text-center">
      <p class="text-gray-400 mb-4">私人漫游未启动</p>
      <button
        @click="startFm"
        class="px-6 py-2 bg-white/10 hover:bg-white/20 rounded-full transition"
      >
        开始漫游
      </button>
    </div>

    <!-- 歌曲信息展示 -->
    <template v-else>
      <!-- 专辑封面 -->
      <img
        :src="currentSong.al?.picUrl || currentSong.album?.picUrl"
        class="w-80 h-80 rounded-3xl object-cover shadow-2xl mb-8"
      />

      <!-- 歌曲名和艺术家 -->
      <h1 class="text-3xl font-bold mb-2">{{ currentSong.name }}</h1>
      <p class="text-lg text-gray-400 mb-8">
        {{ artists }}
      </p>

      <!-- 控制按钮 -->
      <div class="flex items-center gap-8">
        <button
          @click="togglePlay"
          class="w-16 h-16 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 transition border border-white/20"
        >
          <!-- 暂停图标 -->
          <svg v-if="player.playing" width="28" height="28" viewBox="0 0 16 16" fill="currentColor">
            <rect x="3" y="2" width="3" height="12" rx="0.5" />
            <rect x="10" y="2" width="3" height="12" rx="0.5" />
          </svg>
          <!-- 播放图标 -->
          <svg v-else width="28" height="28" viewBox="0 0 16 16" fill="currentColor">
            <path d="M4 2.5v11l9-5.5z" />
          </svg>
        </button>
        <button
          @click="nextSong"
          class="text-3xl text-gray-400 hover:text-white transition"
        >
          ⏭
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { usePlayerStore } from '../stores/player';
import { invoke } from '@tauri-apps/api/core';

const player = usePlayerStore();

// 当前正在播放的歌曲（如果处于FM模式，则显示当前歌曲）
const currentSong = computed(() => {
  // FM 模式下直接显示正在播放的歌曲（可能是FM歌曲）
  if (player.isFmMode && player.currentSong) {
    return player.currentSong;
  }
  return null;
});

const artists = computed(() => {
  if (!currentSong.value) return '';
  return currentSong.value.ar?.map((a: any) => a.name).join(' / ') ||
         currentSong.value.artists?.map((a: any) => a.name).join(' / ') || '';
});

// 进入页面时，如果FM未启动，自动开始
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

function normalizeSong(song: any) {
  const normalized = { ...song };
  if (!normalized.al?.picUrl && normalized.album?.picUrl) {
    normalized.al = { ...normalized.al, picUrl: normalized.album.picUrl };
  }
  if (!normalized.ar || normalized.ar.length === 0) {
    normalized.ar = normalized.artists || [];
  }
  return normalized;
}

async function togglePlay() {
  if (player.playing) {
    await invoke('pause_audio');
  } else {
    if (player.currentSong) {
      // 恢复播放
      await invoke('resume_audio');
    } else {
      await startFm();
    }
  }
}

async function nextSong() {
  await startFm();
}
</script>