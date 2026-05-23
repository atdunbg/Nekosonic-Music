<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>

    <div v-if="artist" class="flex gap-6 mb-8">
      <img :src="artist.cover" class="w-44 h-44 rounded-xl object-cover shadow-lg flex-shrink-0" />
      <div class="flex flex-col justify-between min-w-0">
        <div>
          <h1 class="text-2xl font-bold leading-tight">{{ artist.name }}</h1>
          <p class="text-xs text-content-3 mt-2">
            {{ formatPlayCount(artist.followeds || 0) }} 粉丝 · {{ artist.musicSize || 0 }} 首歌曲
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

    <div class="flex gap-2 mb-6">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        @click="activeTab = tab.key"
        class="px-4 py-1.5 rounded-full text-sm transition"
        :class="activeTab === tab.key ? 'bg-accent text-white' : 'bg-subtle text-content-2 hover:bg-muted'"
      >
        {{ tab.label }}
      </button>
    </div>

    <div v-if="loading" class="text-content-2">加载中...</div>

    <template v-else>
      <div v-if="activeTab === 'songs'" class="space-y-1">
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

      <div v-if="activeTab === 'albums'" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        <div
          v-for="album in albums"
          :key="album.id"
          @click="router.push({ name: 'album', params: { id: album.id } })"
          class="bg-subtle rounded-xl overflow-hidden hover:bg-muted transition cursor-pointer"
        >
          <img :src="album.picUrl" class="w-full aspect-square object-cover" />
          <div class="p-3">
            <p class="text-sm font-medium truncate">{{ album.name }}</p>
            <p class="text-xs text-content-2 mt-1">{{ formatDate(album.publishTime) }}</p>
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'desc'" class="max-w-2xl">
        <p class="text-sm text-content-2 leading-relaxed whitespace-pre-wrap">{{ briefDesc }}</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { formatPlayCount, formatDate } from '../utils/format';
import { normalizeSong, type Song } from '../utils/song';
import SongListItem from '../components/SongListItem.vue';

const route = useRoute();
const router = useRouter();
const player = usePlayerStore();

const artist = ref<any>(null);
const songs = ref<Song[]>([]);
const albums = ref<any[]>([]);
const briefDesc = ref('');
const loading = ref(true);
const activeTab = ref('songs');

const tabs = [
  { key: 'songs', label: '热门歌曲' },
  { key: 'albums', label: '专辑' },
  { key: 'desc', label: '简介' },
];

async function fetchArtist(id: number) {
  loading.value = true;
  artist.value = null;
  songs.value = [];
  albums.value = [];
  briefDesc.value = '';
  try {
    const [detailStr, songsStr, albumStr, descStr] = await Promise.all([
      invoke('artist_detail', { id }) as Promise<string>,
      invoke('artist_songs', { query: { id, order: 'hot', limit: 50, offset: 0 } }) as Promise<string>,
      invoke('artist_album', { id, limit: 30, offset: 0 }) as Promise<string>,
      invoke('artist_desc', { id }) as Promise<string>,
    ]);
    const detailData = JSON.parse(detailStr);
    artist.value = detailData.artist;
    const songsData = JSON.parse(songsStr);
    songs.value = (songsData.songs || []).map(normalizeSong);
    const albumData = JSON.parse(albumStr);
    albums.value = albumData.hotAlbums || [];
    const descData = JSON.parse(descStr);
    briefDesc.value = descData.briefDesc || '';
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  fetchArtist(Number(route.params.id));
});

watch(() => route.params.id, (newId) => {
  if (newId) fetchArtist(Number(newId));
});

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}
</script>
