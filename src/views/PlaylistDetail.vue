<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>

    <div v-if="playlist" class="flex gap-6 mb-8">
      <img :src="playlist.coverImgUrl" class="w-44 h-44 rounded-xl object-cover shadow-lg flex-shrink-0" />
      <div class="flex flex-col justify-between min-w-0">
        <div>
          <h1 class="text-2xl font-bold leading-tight">{{ playlist.name }}</h1>
          <div v-if="playlist.creator" class="flex items-center gap-2 mt-2">
            <img :src="playlist.creator.avatarUrl" class="w-5 h-5 rounded-full" />
            <span class="text-sm text-content-2">{{ playlist.creator.nickname }}</span>
          </div>
          <p class="text-sm text-content-2 mt-2 line-clamp-2">{{ playlist.description }}</p>
          <p class="text-xs text-content-3 mt-2">
            {{ playlist.trackCount }} 首歌曲 · 播放 {{ formatPlayCount(playlist.playCount) }} 次
          </p>
        </div>
        <div class="flex items-center gap-3 mt-4">
          <button
            @click="playAll"
            class="px-5 py-2 bg-accent hover:bg-accent-hover rounded-full text-white font-medium transition flex items-center gap-2"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg>
            播放全部
          </button>
          <button
            v-if="!isOwnPlaylist"
            @click="toggleSubscribe"
            class="px-4 py-2 bg-muted hover:bg-emphasis rounded-full text-sm transition flex items-center gap-2"
            :class="subscribed ? 'text-accent-text' : 'text-content/70'"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path v-if="subscribed" d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z"/>
              <path v-else d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z"/>
            </svg>
            {{ subscribed ? '已收藏' : '收藏歌单' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="loading" class="text-content-2">加载中...</div>

    <div v-else class="space-y-1">
      <div
        v-for="(song, index) in songs"
        :key="song.id"
        @click="playSingle(song)"
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
        <button @click.stop="download.downloadSong(song)" class="text-content-3 hover:text-accent-text transition flex-shrink-0" :title="download.isDownloaded(song.id) ? '已下载' : '下载'">
          <svg v-if="download.isDownloading(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
          <svg v-else-if="download.isDownloaded(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent-text"><polyline points="20 6 9 17 4 12"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </button>
        <span class="text-xs text-content-3">{{ formatDuration(song.dt) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { useDownload } from '../composables/useDownload';
import { showToast } from '../composables/useToast';
import { formatDuration, formatPlayCount } from '../utils/format';

const route = useRoute();
const player = usePlayerStore();
const userStore = useUserStore();
const download = useDownload();

const playlist = ref<any>(null);
const songs = ref<any[]>([]);
const loading = ref(true);
const subscribed = ref(false);

const isOwnPlaylist = computed(() => {
  if (!playlist.value || !userStore.user) return false;
  return playlist.value.creator?.userId === userStore.user.userId;
});

async function fetchPlaylist(id: number) {
  loading.value = true;
  playlist.value = null;
  songs.value = [];
  try {
    const jsonStr: string = await invoke('get_playlist_detail', { id });
    const data = JSON.parse(jsonStr);
    playlist.value = data.playlist;
    songs.value = data.playlist.tracks || [];
    subscribed.value = data.playlist.subscribed || false;
  } catch (e) {
    console.error(e);
    showToast('获取歌单详情失败', 'error');
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  fetchPlaylist(Number(route.params.id));
});

watch(() => route.params.id, (newId) => {
  if (newId) fetchPlaylist(Number(newId));
});

function isCurrentSong(songId: number): boolean {
  return player.currentSong?.id === songId;
}

async function playSingle(song: any) {
  const idx = songs.value.findIndex((s: any) => s.id === song.id);
  player.playFromList(songs.value, idx >= 0 ? idx : 0);
}

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}

async function toggleSubscribe() {
  if (!playlist.value) return;
  const newSubscribed = !subscribed.value;
  try {
    await invoke('playlist_subscribe', { query: { id: Number(playlist.value.id), subscribe: newSubscribed } });
    subscribed.value = newSubscribed;
    showToast(subscribed.value ? '已收藏歌单' : '已取消收藏', 'success');
  } catch {
    showToast('操作失败，请稍后重试', 'error');
  }
}
</script>
