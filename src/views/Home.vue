<template>
  <div class="p-8 text-white">
    <!-- 第一行：每日推荐 & 私人漫游 卡片 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-10">
      <!-- 每日推荐 -->
      <div
        class="h-48 bg-gradient-to-br from-pink-600 to-purple-700 rounded-3xl overflow-hidden relative cursor-pointer group"
        @click="goDaily"
      >
        <div class="absolute inset-0 bg-black/20 group-hover:bg-black/10 transition"></div>
        <div class="relative z-10 p-6 flex flex-col justify-between h-full">
          <div>
            <p class="text-xs text-white/60 mb-1">📅 {{ todayStr }}</p>
            <h2 class="text-2xl font-bold">每日推荐</h2>
          </div>
          <p class="text-xs text-white/60">根据你的口味生成，每天 6:00 更新</p>
        </div>
        <div class="absolute right-4 top-1/2 -translate-y-1/2 text-6xl opacity-20">🎧</div>
      </div>

      <!-- 私人漫游 卡片 -->
       <!-- 私人漫游 卡片 -->
<div
  class="h-48 bg-gradient-to-br from-blue-600 to-cyan-500 rounded-3xl overflow-hidden relative group select-none"
  @click="!userStore.isLoggedIn ? goLogin() : null"
>
  <!-- 模糊封面层（仅在有歌曲且有封面时显示，低透明度模糊） -->
  <div
    v-if="player.fmSong && fmCoverUrl"
    class="absolute inset-0 bg-cover bg-center opacity-30 blur-md scale-110"
    :style="{ backgroundImage: `url(${fmCoverUrl})` }"
  ></div>
  <!-- 遮罩 -->
  <div class="absolute inset-0 bg-black/30 group-hover:bg-black/20 transition"></div>

  <!-- 内容 -->
  <div class="relative z-10 h-full">
    <!-- 未登录 -->
    <div v-if="!userStore.isLoggedIn" class="flex flex-col items-center justify-center h-full">
      <p class="text-xs text-white/60 mb-1">🌀 一键探索</p>
      <h2 class="text-2xl font-bold">私人漫游</h2>
      <p class="text-xs text-white/60 mt-2">登录后即可开启沉浸式音乐探索</p>
    </div>

    <!-- 登录后：无歌曲 → 垂直居中播放按钮 -->
    <div
      v-else-if="!player.fmSong"
      class="flex flex-col items-center justify-center h-full gap-3 cursor-pointer"
      @click.stop="startFmPlay"
    >
      <p class="text-xs text-white/60">🌀 一键探索</p>
      <h2 class="text-2xl font-bold">私人漫游</h2>
      <button
        class="w-12 h-12 flex items-center justify-center rounded-full bg-white/20 hover:bg-white/30 transition mt-2"
      >
        <svg width="22" height="22" viewBox="0 0 16 16" fill="currentColor" class="text-white">
          <path d="M4 2.5v11l9-5.5z" />
        </svg>
      </button>
    </div>

    <!-- 有歌曲 → 横向布局：左侧信息，右侧按钮 -->
    <div v-else class="flex items-center justify-between h-full px-6 cursor-pointer" @click.stop="player.toggleFm">
      <!-- 左侧：封面 + 歌曲信息 -->
      <div class="flex items-center gap-3 min-w-0">
        <img :src="fmCoverUrl" class="w-14 h-14 rounded-xl object-cover flex-shrink-0" />
        <div class="min-w-0">
          <p class="text-sm font-semibold truncate">{{ fmDisplayName }}</p>
          <p class="text-xs text-white/70 truncate">{{ fmDisplayArtists }}</p>
        </div>
      </div>
      <!-- 右侧：控制按钮 -->
      <div class="flex items-center gap-3 ml-4">
        <button @click.stop="player.toggleFm"
          class="w-10 h-10 flex items-center justify-center rounded-full bg-white/20 hover:bg-white/30 transition"
        >
          <svg v-if="player.fmPlaying" width="18" height="18" viewBox="0 0 16 16" fill="currentColor" class="text-white">
            <rect x="3" y="2" width="3" height="12" rx="0.5" />
            <rect x="10" y="2" width="3" height="12" rx="0.5" />
          </svg>
          <svg v-else width="18" height="18" viewBox="0 0 16 16" fill="currentColor" class="text-white">
            <path d="M4 2.5v11l9-5.5z" />
          </svg>
        </button>
        <button @click.stop="player.nextFm" class="text-xl text-white/80 hover:text-white transition">⏭</button>
      </div>
    </div>
  </div>

  <div class="absolute right-4 top-1/2 -translate-y-1/2 text-6xl opacity-20 pointer-events-none">🌊</div>
</div>

    </div>

    <!-- 第二行：为你推荐（需登录） -->
    <div v-if="userStore.isLoggedIn && recPlaylists.length" class="mb-10">
      <h2 class="text-xl font-semibold mb-4">🎯 为你推荐</h2>
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        <div v-for="pl in recPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
          class="bg-white/5 rounded-xl overflow-hidden hover:bg-white/10 transition cursor-pointer">
          <img :src="pl.picUrl" class="w-full aspect-square object-cover" />
          <div class="p-3">
            <p class="text-sm font-medium truncate">{{ pl.name }}</p>
            <p class="text-xs text-gray-400 mt-1">{{ pl.copywriter || '' }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 第三行：热门歌单（排行榜） -->
    <div>
      <h2 class="text-xl font-semibold mb-4">📈 热门歌单</h2>
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        <div v-for="pl in rankPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
          class="bg-white/5 rounded-xl overflow-hidden hover:bg-white/10 transition cursor-pointer backdrop-blur-sm">
          <img :src="pl.coverImgUrl" class="w-full aspect-square object-cover" />
          <div class="p-3">
            <p class="text-sm font-medium truncate">{{ pl.name }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useUserStore } from '../stores/user';
import { usePlayerStore } from '../stores/player';

const player = usePlayerStore();
const router = useRouter();
const userStore = useUserStore();

const rankPlaylists = ref<any[]>([]);
const recPlaylists = ref<any[]>([]);
const todayStr = ref('');
const RANK_IDS = [3778678, 3779629, 19723756, 2884035];

import { computed } from 'vue';


const fmCoverUrl = computed(() => {
  return player.fmSong?.al?.picUrl || player.fmSong?.album?.picUrl || '';
});
const fmDisplayName = computed(() => player.fmSong?.name || '私人漫游');
const fmDisplayArtists = computed(() => {
  if (!player.fmSong) return '';
  return player.fmSong.ar?.map((a: any) => a.name).join(' / ') ||
         player.fmSong.artists?.map((a: any) => a.name).join(' / ') || '';
});


// 首次点击播放按钮：开始 FM 并播放
async function startFmPlay() {
  // 如果还没加载过 FM，或者之前加载了但被停止了，重新加载
  if (!player.fmSong) {
    await player.loadFm(); // loadFm 内部会设置 fmSong 并播放
  } else {
    // 已有歌曲但未播放状态（比如之前暂停/停止了），直接播放
    await player.toggleFm();
  }
}

onMounted(async () => {
  const d = new Date();
  todayStr.value = `${d.getMonth() + 1}月${d.getDate()}日`;

  // 排行榜
  const results = await Promise.allSettled(
    RANK_IDS.map(id => invoke('get_playlist_detail', { id }))
  );
  rankPlaylists.value = results
    .filter(r => r.status === 'fulfilled')
    .map((r: any) => {
      const data = JSON.parse(r.value);
      return data.playlist;
    })
    .filter(Boolean);

  // 推荐歌单（需登录）
  if (userStore.isLoggedIn) {
    try {
      const json = await invoke('recommend_resource');
      const data = JSON.parse(json as string);
      recPlaylists.value = data.recommend || [];
    } catch { }
  }
});

function goDaily() {
  router.push('/daily');
}

function goPlaylist(id: number) {
  router.push({ name: 'playlist', params: { id } });
}

function goLogin() {
  router.push('/login');
}
</script>