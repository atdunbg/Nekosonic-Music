<template>
  <div class="p-8 text-content">
    <PageHeader />

    <!-- 头部骨架 -->
    <div v-if="!album && albumLoading" class="flex gap-6 mb-8">
      <div class="w-44 h-44 rounded-xl bg-muted animate-pulse flex-shrink-0"></div>
      <div class="flex-1 space-y-3">
        <div class="h-7 bg-muted rounded w-1/2 animate-pulse"></div>
        <div class="h-4 bg-muted rounded w-1/3 animate-pulse"></div>
        <div class="h-4 bg-muted rounded w-1/4 animate-pulse"></div>
        <div class="h-10 w-28 bg-muted rounded-full animate-pulse mt-4"></div>
      </div>
    </div>

    <!-- 头部信息 -->
    <div v-else-if="album" class="flex gap-6 mb-8">
      <img :src="album.picUrl" class="w-44 h-44 rounded-xl object-cover shadow-lg flex-shrink-0" />
      <div class="flex flex-col justify-between min-w-0">
        <div>
          <h1 class="text-2xl font-bold leading-tight">{{ album.name }}</h1>
          <div v-if="album.artists?.length" class="flex flex-wrap items-center gap-x-1 gap-y-0.5 mt-2 text-sm text-content-2">
            <template v-for="(ar, idx) in album.artists" :key="ar.id">
              <span v-if="(idx as number) > 0" class="text-content-3">/</span>
              <span
                class="hover:text-accent-text cursor-pointer transition whitespace-nowrap"
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
            <IconPlay class="w-4 h-4 fill-current" />
            播放全部
          </button>
        </div>
      </div>
    </div>

    <!-- 加载失败 -->
    <div v-if="loadError" class="flex flex-col items-center justify-center py-16 gap-3">
      <p class="text-content-2 text-sm">加载失败</p>
      <button @click="fetchAlbum(Number(route.params.id), true)" class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">重试</button>
    </div>

    <!-- 歌曲列表骨架 -->
    <div v-else-if="songsLoading" class="space-y-1">
      <div v-for="i in 6" :key="i" class="flex items-center gap-3 px-3 py-2">
        <div class="w-12 h-12 bg-muted rounded animate-pulse flex-shrink-0"></div>
        <div class="flex-1 space-y-2">
          <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
          <div class="h-3 bg-muted rounded w-1/3 animate-pulse"></div>
        </div>
      </div>
    </div>

    <!-- 歌曲列表 -->
    <div v-else-if="songs.length" class="space-y-1">
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
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onActivated } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { normalizeSong, type Song } from '../utils/song';
import { formatDate } from '../utils/format';
import { pageCacheGet, pageCacheSet } from '../composables/usePageCache';
import SongListItem from '../components/SongListItem.vue';
import PageHeader from '../components/PageHeader.vue';
import IconPlay from '~icons/lucide/play';

defineOptions({ name: 'AlbumDetailView' });

const route = useRoute();
const router = useRouter();
const player = usePlayerStore();

const album = ref<any>(null);
const songs = ref<Song[]>([]);
const albumLoading = ref(true);
const songsLoading = ref(false);
const loadError = ref(false);

async function fetchAlbum(id: number, force = false) {
  const cacheKey = `album_${id}`;
  if (!force) {
    const cached = pageCacheGet(cacheKey);
    if (cached) {
      album.value = cached.album;
      songs.value = cached.songs;
      albumLoading.value = false;
      songsLoading.value = false;
      loadError.value = false;
      return;
    }
  }

  albumLoading.value = true;
  songsLoading.value = true;
  loadError.value = false;
  album.value = null;
  songs.value = [];
  try {
    const jsonStr: string = await MusicApi.albumDetail(id);
    const data = JSON.parse(jsonStr);
    album.value = data.album;
    albumLoading.value = false;
    songs.value = (data.songs || []).map(normalizeSong);
    songsLoading.value = false;
    pageCacheSet(cacheKey, { album: album.value, songs: songs.value });
  } catch (e) {
    console.error(e);
    loadError.value = true;
    albumLoading.value = false;
    songsLoading.value = false;
  }
}

onMounted(() => {
  fetchAlbum(Number(route.params.id));
});

watch(() => route.params.id, (newId) => {
  if (newId && route.name === 'album') fetchAlbum(Number(newId));
});

onActivated(() => {
  if (loadError.value) fetchAlbum(Number(route.params.id), true);
});

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}
</script>
