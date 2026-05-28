<template>
  <div class="p-8 text-content">
    <!-- 第一行：每日推荐 & 私人漫游 卡片 -->
    <div class="grid grid-cols-2 gap-6 mb-10">
      <!-- 每日推荐 -->
      <div
        class="h-48 bg-gradient-to-br from-pink-600 to-purple-700 rounded-3xl overflow-hidden relative cursor-pointer group"
        @click="goDaily"
      >
        <div class="absolute inset-0 bg-black/20 group-hover:bg-black/10 transition"></div>
        <div class="relative z-10 p-6 flex flex-col justify-between h-full">
          <div>
            <p class="text-xs text-white/60 mb-1">📅 {{ todayStr }}</p>
            <h2 class="text-2xl font-bold text-white">每日推荐</h2>
          </div>
          <p class="text-xs text-white/60">根据你的口味生成，每天凌晨更新</p>
        </div>
        <div class="absolute right-4 top-1/2 -translate-y-1/2 text-6xl opacity-20">🎧</div>
      </div>

      <!-- 私人漫游 卡片 -->
      <div
        class="h-48 rounded-3xl overflow-hidden relative group select-none cursor-pointer"
        :class="player.fmSong && fmCoverUrl ? '' : 'bg-gradient-to-br from-indigo-600 via-blue-600 to-cyan-500'"
        @click="onFmCardClick"
      >
        <div
          v-if="player.fmSong && fmCoverUrl"
          class="absolute inset-0 bg-cover bg-center scale-110"
          :style="{ backgroundImage: `url(${fmCoverUrl})` }"
        ></div>
        <div class="absolute inset-0 bg-gradient-to-t from-black/70 via-black/30 to-black/10 group-hover:from-black/60 transition"></div>

        <div class="relative z-10 h-full flex flex-col justify-between p-6">
          <div class="flex items-center gap-2">
            <IconRadio class="w-4 h-4 text-white/50" />
            <span class="text-xs text-white/50 font-medium">私人漫游</span>
          </div>

          <div class="flex items-end justify-between gap-4">
            <div class="min-w-0 flex-1">
              <h2 class="text-xl font-bold text-white" v-if="!player.fmSong && userStore.isLoggedIn">发现新音乐</h2>
              <h2 class="text-xl font-bold text-white" v-else-if="!userStore.isLoggedIn">私人漫游</h2>
              <h2 class="text-lg font-bold truncate text-white" v-else>{{ fmDisplayName }}</h2>
              <p v-if="!userStore.isLoggedIn" class="text-xs text-white/50 mt-1">登录后开启沉浸式音乐探索</p>
              <p v-else-if="!player.fmSong" class="text-xs text-white/50 mt-1">根据你的喜好，为你推荐意想不到的好歌</p>
              <p v-else class="text-xs text-white/60 truncate mt-1">{{ fmDisplayArtists }}</p>
            </div>
            <div class="flex items-center gap-2 flex-shrink-0">
              <button v-if="userStore.isLoggedIn && !player.fmSong"
                @click.stop="startFmPlay"
                class="w-10 h-10 flex items-center justify-center rounded-full bg-white/15 hover:bg-white/25 backdrop-blur-sm transition">
                <IconPlay class="w-4 h-4 fill-current text-white" />
              </button>
              <template v-if="player.fmSong">
                <button @click.stop="player.toggleFm"
                  class="w-10 h-10 flex items-center justify-center rounded-full bg-white/15 hover:bg-white/25 backdrop-blur-sm transition">
                  <IconPause v-if="player.fmPlaying" class="w-[18px] h-[18px] fill-current text-white" />
                  <IconPlay v-else class="w-[18px] h-[18px] fill-current text-white" />
                </button>
                <button @click.stop="player.nextFm"
                  class="w-8 h-8 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 backdrop-blur-sm transition">
                  <IconSkipForward class="w-[14px] h-[14px] text-white" />
                </button>
              </template>
            </div>
          </div>
        </div>
      </div>

    </div>

    <!-- 第二行：为你推荐（需登录） -->
    <div v-if="userStore.isLoggedIn && recPlaylists.length" class="mb-10">
      <h2 class="text-xl font-semibold mb-4">🎯 为你推荐</h2>
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-4">
        <div v-for="pl in recPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
          class="bg-subtle rounded-xl overflow-hidden hover:bg-muted transition cursor-pointer max-w-[220px] justify-self-center w-full">
          <img :src="pl.picUrl" class="w-full aspect-square object-cover" />
          <div class="p-3">
            <p class="text-sm font-medium truncate">{{ pl.name }}</p>
            <p class="text-xs text-content-2 mt-1">{{ pl.copywriter || '' }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 第三行：热门歌单（排行榜） -->
    <div>
      <h2 class="text-xl font-semibold mb-4">📈 热门歌单</h2>
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-4">
        <div v-for="pl in rankPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
          class="bg-subtle rounded-xl overflow-hidden hover:bg-muted transition cursor-pointer backdrop-blur-sm max-w-[220px] justify-self-center w-full">
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
import { ref, onMounted, onActivated, watch } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useUserStore } from '../stores/user';
import { usePlayerStore } from '../stores/player';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { useOnlineStatus } from '../composables/useOnlineStatus';
import { getCoverUrl } from '../utils/song';

defineOptions({ name: 'HomeView' });

const player = usePlayerStore();
const router = useRouter();
const userStore = useUserStore();
const { isOnline } = useOnlineStatus();

const rankPlaylists = ref<any[]>([]);
const recPlaylists = ref<any[]>([]);
const todayStr = ref('');
const RANK_IDS = [3778678, 3779629, 19723756, 2884035];

import { computed } from 'vue';
import IconRadio from '~icons/lucide/radio';
import IconPlay from '~icons/lucide/play';
import IconPause from '~icons/lucide/pause';
import IconSkipForward from '~icons/lucide/skip-forward';


const fmCoverUrl = computed(() => {
  return getCoverUrl(player.fmSong) || '';
});
const fmDisplayName = computed(() => player.fmSong?.name || '私人漫游');
const fmDisplayArtists = computed(() => {
  if (!player.fmSong) return '';
  return player.fmSong.ar?.map((a: { name: string }) => a.name).join(' / ') || '';
});


async function startFmPlay() {
  if (!player.fmSong) {
    await player.loadFm();
  } else {
    await player.toggleFm();
  }
}

function onFmCardClick() {
  if (!userStore.isLoggedIn) {
    goLogin();
    return;
  }
  if (!player.fmSong) {
    startFmPlay();
    return;
  }
  player.openRoamDrawer();
}

async function loadData() {
  const cached = pageCacheGet('home');
  if (cached) {
    rankPlaylists.value = cached.rankPlaylists || [];
    recPlaylists.value = cached.recPlaylists || [];
    return;
  }

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

  if (userStore.isLoggedIn) {
    try {
      const json = await invoke('recommend_resource');
      const data = JSON.parse(json as string);
      recPlaylists.value = data.recommend || [];
    } catch { /* 忽略 */ }
  }

  pageCacheSet('home', { rankPlaylists: rankPlaylists.value, recPlaylists: recPlaylists.value });
}

onMounted(async () => {
  const d = new Date();
  todayStr.value = `${d.getMonth() + 1}月${d.getDate()}日`;
  await loadData();
});

onActivated(() => {
  if (pageCacheIsStale('home')) loadData();
});

watch(isOnline, (val, old) => {
  if (val && !old && rankPlaylists.value.length === 0 && recPlaylists.value.length === 0) {
    pageCacheInvalidate('home');
    loadData();
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