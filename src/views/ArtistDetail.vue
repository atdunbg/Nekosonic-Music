<template>
  <div class="p-8 text-content">
    <PageHeader />

    <!-- 头部骨架 -->
    <div v-if="!artist && !songs.length && !albums.length" class="flex gap-6 mb-4">
      <div class="w-44 h-44 rounded-full bg-muted animate-pulse flex-shrink-0"></div>
      <div class="flex-1 space-y-3">
        <div class="h-7 bg-muted rounded w-1/3 animate-pulse"></div>
        <div class="h-4 bg-muted rounded w-1/4 animate-pulse"></div>
        <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
        <div class="flex gap-3 mt-4">
          <div class="h-10 w-28 bg-muted rounded-full animate-pulse"></div>
          <div class="h-10 w-20 bg-muted rounded-full animate-pulse"></div>
        </div>
      </div>
    </div>

    <div v-else-if="loadError" class="flex flex-col items-center justify-center py-16 gap-3">
      <p class="text-content-2 text-sm">加载失败</p>
      <button @click="fetchArtist(Number(route.params.id), true)" class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">重试</button>
    </div>

    <template v-if="!loadError">
      <!-- 头部：头像 + 简介 -->
      <div v-if="artist || songs.length || albums.length" class="flex gap-6 mb-4">
        <img v-if="artistCover" :src="artistCover" class="w-44 h-44 rounded-full object-cover shadow-lg flex-shrink-0" />
        <div v-else class="w-44 h-44 rounded-full bg-muted flex items-center justify-center flex-shrink-0">
          <IconMusic class="w-12 h-12 text-content-4" />
        </div>
        <div class="flex flex-col min-w-0 flex-1">
          <div>
            <h1 class="text-2xl font-bold leading-tight">{{ artistName }}</h1>
            <p v-if="artistFollowers || artist?.musicSize" class="text-xs text-content-3 mt-1">
              <span v-if="artistFollowers">{{ formatPlayCount(artistFollowers) }} 粉丝</span>
              <span v-if="artistFollowers && artist?.musicSize"> · </span>
              <span v-if="artist?.musicSize">{{ artist.musicSize }} 首歌曲</span>
            </p>
          </div>
          <div v-if="briefDesc" class="mt-3">
            <p
              ref="descEl"
              class="text-sm text-content-2 leading-relaxed whitespace-pre-wrap overflow-hidden"
              style="max-height: 4.5em"
            >{{ briefDesc }}</p>
            <button
              v-if="descOverflow"
              @click="showDescModal = true"
              class="inline-flex items-center gap-1 text-xs text-accent-text hover:text-accent-text/80 mt-1.5 px-2 py-0.5 rounded-full bg-accent-text/10 transition"
            >
              <IconChevronDown class="w-3 h-3" />
              查看完整介绍
            </button>
          </div>
          <div class="flex items-center gap-3 mt-auto pt-4">
            <button
              @click="playAll"
              class="px-5 py-2 bg-accent hover:bg-accent-hover rounded-full text-white font-medium transition flex items-center gap-2"
            >
              <IconPlay class="w-4 h-4 fill-current" />
              播放全部
            </button>
            <button
              @click="toggleFollow"
              :disabled="followLoading"
              class="px-5 py-2 rounded-full font-medium transition flex items-center gap-2"
              :class="isFollowed
                ? 'bg-subtle text-content-2 hover:bg-muted'
                : 'bg-accent/15 text-accent-text hover:bg-accent/25'"
            >
              {{ isFollowed ? '已关注' : '关注' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 简介弹窗 -->
      <Teleport to="body">
        <div v-if="showDescModal" class="fixed inset-0 z-50 flex items-center justify-center" @click.self="showDescModal = false">
          <div class="absolute inset-0 bg-black/50" @click="showDescModal = false"></div>
          <div class="relative bg-surface rounded-2xl shadow-2xl max-w-lg w-full mx-4 max-h-[70vh] flex flex-col">
            <div class="flex items-center justify-between p-5 border-b border-line-2">
              <h2 class="text-lg font-semibold">{{ artistName }} 的介绍</h2>
              <button @click="showDescModal = false" class="text-content-3 hover:text-content transition">
                <IconX class="w-5 h-5" />
              </button>
            </div>
            <div class="p-5 overflow-y-auto text-sm text-content-2 leading-relaxed whitespace-pre-wrap">{{ briefDesc }}</div>
          </div>
        </div>
      </Teleport>

      <!-- 内容区：热门歌曲 / 专辑 -->
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

      <!-- 歌曲列表 -->
      <div v-if="activeTab === 'songs'">
        <div v-if="songsLoading" class="space-y-1">
          <div v-for="i in 6" :key="i" class="flex items-center gap-3 px-3 py-2">
            <div class="w-12 h-12 bg-muted rounded animate-pulse flex-shrink-0"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
              <div class="h-3 bg-muted rounded w-1/3 animate-pulse"></div>
            </div>
          </div>
        </div>
        <VirtualSongList
          v-else-if="songs.length"
          :songs="songs"
          :current-song-id="player.currentSong?.id"
          show-index
          show-like
          show-download
          show-menu
          show-duration
          show-playing-overlay
          @song-click="(_s, i) => player.playFromList(songs, i)"
        />
      </div>

      <!-- 专辑列表 -->
      <div v-if="activeTab === 'albums'">
        <div v-if="albumsLoading" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          <div v-for="i in 8" :key="i" class="bg-muted rounded-xl animate-pulse">
            <div class="w-full aspect-square"></div>
            <div class="p-3 space-y-2">
              <div class="h-4 bg-subtle rounded w-3/4"></div>
              <div class="h-3 bg-subtle rounded w-1/2"></div>
            </div>
          </div>
        </div>
        <div v-else-if="albums.length" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
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
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onActivated, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { formatPlayCount, formatDate } from '../utils/format';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet } from '../composables/usePageCache';
import VirtualSongList from '../components/VirtualSongList.vue';
import PageHeader from '../components/PageHeader.vue';
import IconPlay from '~icons/lucide/play';
import IconMusic from '~icons/lucide/music';
import IconX from '~icons/lucide/x';
import IconChevronDown from '~icons/lucide/chevron-down';

defineOptions({ name: 'ArtistDetailView' });

const route = useRoute();
const router = useRouter();
const player = usePlayerStore();

const artist = ref<any>(null);
const songs = ref<Song[]>([]);
const albums = ref<any[]>([]);
const briefDesc = ref('');
const loadError = ref(false);
const songsLoading = ref(false);
const albumsLoading = ref(false);
const activeTab = ref('songs');
const showDescModal = ref(false);
const descOverflow = ref(false);
const descEl = ref<HTMLElement | null>(null);
const isFollowed = ref(false);
const followLoading = ref(false);

const tabs = [
  { key: 'songs', label: '热门歌曲' },
  { key: 'albums', label: '专辑' },
];

const artistName = computed(() => {
  if (artist.value?.name) return artist.value.name;
  if (songs.value.length > 0 && songs.value[0].ar?.length > 0) return songs.value[0].ar[0].name;
  if (albums.value.length > 0) return albums.value[0].artist?.name || albums.value[0].artists?.[0]?.name || '';
  return '未知歌手';
});

const artistCover = computed(() => {
  if (artist.value?.cover) return artist.value.cover;
  if (artist.value?.picUrl) return artist.value.picUrl;
  if (artist.value?.img1v1Url) return artist.value.img1v1Url;
  return '';
});

const artistFollowers = computed(() => {
  if (!artist.value) return 0;
  return artist.value.followeds || artist.value.followCount || artist.value.fans || 0;
});

function checkDescOverflow() {
  nextTick(() => {
    if (descEl.value) {
      descOverflow.value = descEl.value.scrollHeight > descEl.value.clientHeight + 2;
    }
  });
}

async function fetchArtist(id: number, force = false) {
  const cacheKey = `artist_${id}`;
  if (!force) {
    const cached = pageCacheGet(cacheKey);
    if (cached) {
      artist.value = cached.artist;
      songs.value = cached.songs;
      albums.value = cached.albums;
      briefDesc.value = cached.briefDesc;
      loadError.value = false;
      checkDescOverflow();
      return;
    }
  }

  loadError.value = false;
  artist.value = null;
  songs.value = [];
  albums.value = [];
  briefDesc.value = '';

  const loadDetail = async () => {
    try {
      const jsonStr = await MusicApi.artistDetail(id);
      const data = JSON.parse(jsonStr);
      const a = data.artist || data.data?.artist || data;
      artist.value = a;
    } catch { /* 忽略 */ }
  };

  const loadFollowStatus = async () => {
    try {
      const jsonStr = await MusicApi.artistSublist(100, 0);
      const data = JSON.parse(jsonStr);
      const list = data.data || [];
      isFollowed.value = list.some((item: any) => item.id === id);
    } catch { /* 忽略 */ }
  };

  const loadSongs = async () => {
    songsLoading.value = true;
    try {
      const jsonStr = await MusicApi.artistSongs({ id, order: 'hot', limit: 50, offset: 0 });
      const data = JSON.parse(jsonStr);
      songs.value = (data.songs || []).map(normalizeSong);
    } catch { /* 忽略 */ }
    finally { songsLoading.value = false; }
  };

  const loadAlbums = async () => {
    albumsLoading.value = true;
    try {
      const jsonStr = await MusicApi.artistAlbum(id, 30, 0);
      const data = JSON.parse(jsonStr);
      albums.value = data.hotAlbums || [];
    } catch { /* 忽略 */ }
    finally { albumsLoading.value = false; }
  };

  const loadDesc = async () => {
    try {
      const jsonStr = await MusicApi.artistDesc(id);
      const data = JSON.parse(jsonStr);
      if (data.briefDesc) {
        briefDesc.value = data.briefDesc;
      } else if (Array.isArray(data.introduction) && data.introduction.length > 0) {
        briefDesc.value = data.introduction.map((item: any) => item.txt || '').filter(Boolean).join('\n');
      }
      checkDescOverflow();
    } catch { /* 忽略 */ }
  };

  await Promise.allSettled([loadDetail(), loadSongs(), loadAlbums(), loadDesc(), loadFollowStatus()]);

  if (!artist.value && !songs.value.length && !albums.value.length && !briefDesc.value) {
    loadError.value = true;
    return;
  }

  pageCacheSet(cacheKey, { artist: artist.value, songs: songs.value, albums: albums.value, briefDesc: briefDesc.value });
}

onMounted(() => {
  fetchArtist(Number(route.params.id));
});

watch(() => route.params.id, (newId) => {
  if (newId && route.name === 'artist') fetchArtist(Number(newId));
});

onActivated(() => {
  if (loadError.value) fetchArtist(Number(route.params.id), true);
});

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}

async function toggleFollow() {
  const id = Number(route.params.id);
  if (!id || followLoading.value) return;
  followLoading.value = true;
  try {
    await MusicApi.artistSub(id, !isFollowed.value);
    isFollowed.value = !isFollowed.value;
  } catch (e) {
    console.error('关注操作失败', e);
  } finally {
    followLoading.value = false;
  }
}
</script>
