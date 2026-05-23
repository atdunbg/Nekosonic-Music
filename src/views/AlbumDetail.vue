<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>

    <div v-if="album" class="flex gap-6 mb-8">
      <img :src="album.picUrl" class="w-44 h-44 rounded-xl object-cover shadow-lg flex-shrink-0" />
      <div class="flex flex-col justify-between min-w-0">
        <div>
          <h1 class="text-2xl font-bold leading-tight">{{ album.name }}</h1>
          <div v-if="album.artists?.length" class="flex items-center gap-1 mt-2 text-sm text-content-2">
            <template v-for="(ar, idx) in album.artists" :key="ar.id">
              <span v-if="(idx as number) > 0" class="text-content-3">/</span>
              <span
                class="hover:text-accent-text cursor-pointer transition"
                @click="ar.id && router.push({ name: 'artist', params: { id: ar.id } })"
              >{{ ar.name }}</span>
            </template>
          </div>
          <p class="text-xs text-content-3 mt-2">
            {{ formatDate(album.publishTime) }} · {{ songs.length }} 首歌曲
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
        </div>
      </div>
    </div>

    <div v-if="loading" class="text-content-2">加载中...</div>

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
      >
        <template #index="{ index: idx, isCurrent }">
          <div class="w-6 text-right flex-shrink-0 flex items-center justify-end h-5">
            <div v-if="isCurrent" class="flex items-center justify-end">
              <div class="flex items-center gap-[3px] h-4">
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 50%; animation-delay: 0ms"></span>
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 100%; animation-delay: 150ms"></span>
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 35%; animation-delay: 300ms"></span>
              </div>
            </div>
            <template v-else>
              <span class="text-xs text-content-3 group-hover:hidden">{{ idx + 1 }}</span>
              <svg class="hidden group-hover:block text-content" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg>
            </template>
          </div>
        </template>
      </SongListItem>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { normalizeSong, type Song } from '../utils/song';
import { formatDate } from '../utils/format';
import SongListItem from '../components/SongListItem.vue';

const route = useRoute();
const router = useRouter();
const player = usePlayerStore();

const album = ref<any>(null);
const songs = ref<Song[]>([]);
const loading = ref(true);

async function fetchAlbum(id: number) {
  loading.value = true;
  album.value = null;
  songs.value = [];
  try {
    const jsonStr: string = await invoke('album_detail', { id });
    const data = JSON.parse(jsonStr);
    album.value = data.album;
    songs.value = (data.songs || []).map(normalizeSong);
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  fetchAlbum(Number(route.params.id));
});

watch(() => route.params.id, (newId) => {
  if (newId) fetchAlbum(Number(newId));
});

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}
</script>
