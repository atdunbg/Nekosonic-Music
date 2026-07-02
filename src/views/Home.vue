<template>
  <div class="p-8 text-content">
    <!-- 第一行：每日推荐 & 私人漫游 卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-10">
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

    <!-- 推荐歌单（个性化，无需登录也可获取） -->
    <section class="mb-10">
      <SectionHeader
        :title="userStore.isLoggedIn ? '🎯 为你推荐' : '🎵 推荐歌单'"
        :subtitle="userStore.isLoggedIn ? '根据你的口味生成' : '精选好歌单'"
        :icon="IconSparkles"
      />
      <CardGrid
        :items="recPlaylists"
        :loading="recLoading"
        :error="recError"
        error-text="推荐歌单加载失败"
        @retry="fetchRecPlaylists"
      >
        <PlaylistCard
          v-for="pl in recPlaylists"
          :key="pl.id"
          :playlist="pl"
          :is-current-playing="player.currentSource.type === 'playlist' && player.currentSource.id === pl.id && player.playing"
          @click="goPlaylist(pl.id)"
          @play="playPlaylist(pl)"
        />
      </CardGrid>
    </section>

    <!-- 新歌速递 -->
    <section class="mb-10">
      <SectionHeader
        :title="userStore.isLoggedIn ? '🆕 推荐新歌' : '🆕 新歌速递'"
        :subtitle="userStore.isLoggedIn ? '根据你的口味推荐' : '发现最新音乐'"
        :icon="IconTrendingUp"
      />
      <CardGrid
        :items="newSongs"
        :loading="newSongsLoading"
        :error="newSongsError"
        error-text="新歌加载失败"
        :skeleton-count="4"
        grid-class="grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
        @retry="fetchNewSongs"
      >
        <div
          v-for="(song, index) in newSongs"
          :key="song.id"
          @click="playNewSong(index)"
          class="flex items-center gap-3 p-3 rounded-xl bg-subtle hover:bg-muted transition cursor-pointer group"
        >
          <div class="w-12 h-12 rounded-md overflow-hidden flex-shrink-0 relative">
            <img
              v-if="song.al?.picUrl"
              :src="song.al.picUrl"
              class="w-full h-full object-cover"
              loading="lazy"
            />
            <div v-else class="w-full h-full bg-muted flex items-center justify-center">
              <IconMusic class="w-4 h-4 text-content-4" />
            </div>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate text-content flex items-center gap-1">
              <span class="truncate">{{ song.name }}</span>
              <span
                v-for="tag in getSongTags(song)"
                :key="tag.kind"
                class="text-[10px] leading-none px-1 py-0.5 rounded flex-shrink-0 font-medium"
                :class="tag.class"
              >{{ tag.label }}</span>
            </p>
            <p class="text-xs text-content-2 truncate">{{ getArtistDisplay(song) }}</p>
          </div>
          <button
            class="w-8 h-8 flex items-center justify-center rounded-full bg-muted/50 text-content-2 group-hover:bg-accent group-hover:text-white transition flex-shrink-0"
            :title="isCurrentNewSong(song) ? (player.playing ? '暂停' : '播放') : '播放'"
          >
            <IconPause v-if="isCurrentNewSong(song) && player.playing" class="w-3.5 h-3.5 fill-current" />
            <IconPlay v-else class="w-3.5 h-3.5 fill-current ml-0.5" />
          </button>
        </div>
      </CardGrid>
    </section>

    <!-- 热门歌手 -->
    <section class="mb-10">
      <SectionHeader title="🎤 热门歌手" subtitle="人气音乐人" :icon="IconFlame" />
      <CardGrid
        :items="topArtists"
        :loading="artistsLoading"
        :error="artistsError"
        error-text="热门歌手加载失败"
        :skeleton-count="6"
        grid-class="grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8"
        @retry="fetchTopArtists"
      >
        <ArtistCard
          v-for="ar in topArtists"
          :key="ar.id"
          :artist="ar"
          @click="goArtist(ar.id)"
        />
      </CardGrid>
    </section>

    <!-- 新碟上架 -->
    <section class="mb-10">
      <SectionHeader title="💿 新碟上架" subtitle="最新专辑" :icon="IconDisc" />
      <CardGrid
        :items="newAlbums"
        :loading="albumsLoading"
        :error="albumsError"
        error-text="新碟上架加载失败"
        @retry="fetchNewAlbums"
      >
        <AlbumCard
          v-for="al in newAlbums"
          :key="al.id"
          :album="al"
          :is-current-playing="player.currentSource.type === 'album' && player.currentSource.id === al.id && player.playing"
          @click="goAlbum(al.id)"
          @play="playAlbum(al)"
        />
      </CardGrid>
    </section>

    <!-- 热门歌单（排行榜） -->
    <section>
      <SectionHeader title="📈 热门歌单" subtitle="排行榜精选" :icon="IconTrendingUp" />
      <CardGrid
        :items="rankPlaylists"
        :loading="rankLoading"
        :error="rankError"
        error-text="热门歌单加载失败"
        @retry="fetchRankPlaylists"
      >
        <PlaylistCard
          v-for="pl in rankPlaylists"
          :key="pl.id"
          :playlist="pl"
          :is-current-playing="player.currentSource.type === 'playlist' && player.currentSource.id === pl.id && player.playing"
          @click="goPlaylist(pl.id)"
          @play="playPlaylist(pl)"
        />
      </CardGrid>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onActivated, watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import { MusicApi, RecApi } from '../api';
import { useUserStore } from '../stores/user';
import { usePlayerStore } from '../stores/player';
import { useUiStore } from '../stores/ui';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { useOnlineStatus } from '../composables/useOnlineStatus';
import { getCoverUrl, getArtistDisplay, getSongTags, normalizeSong, type Song } from '../utils/song';
import PlaylistCard, { type PlaylistCardData } from '../components/Card/PlaylistCard.vue';
import ArtistCard, { type ArtistCardData } from '../components/Card/ArtistCard.vue';
import AlbumCard, { type AlbumCardData } from '../components/Card/AlbumCard.vue';
import SectionHeader from '../components/Card/SectionHeader.vue';
import CardGrid from '../components/Card/CardGrid.vue';
import IconRadio from '~icons/lucide/radio';
import IconPlay from '~icons/lucide/play';
import IconPause from '~icons/lucide/pause';
import IconSkipForward from '~icons/lucide/skip-forward';
import IconMusic from '~icons/lucide/music';
import IconSparkles from '~icons/lucide/sparkles';
import IconTrendingUp from '~icons/lucide/trending-up';
import IconFlame from '~icons/lucide/flame';
import IconDisc from '~icons/lucide/disc-3';

defineOptions({ name: 'HomeView' });

const player = usePlayerStore();
const ui = useUiStore();
const router = useRouter();
const userStore = useUserStore();
const { isOnline } = useOnlineStatus();

// --- 推荐歌单 ---
const recPlaylists = ref<PlaylistCardData[]>([]);
const recLoading = ref(false);
const recError = ref(false);

// --- 热门歌单（排行榜） ---
const rankPlaylists = ref<PlaylistCardData[]>([]);
const rankLoading = ref(false);
const rankError = ref(false);
const RANK_IDS = [3778678, 3779629, 19723756, 2884035];

// --- 新歌速递 ---
const newSongs = ref<Song[]>([]);
const newSongsLoading = ref(false);
const newSongsError = ref(false);

// --- 热门歌手 ---
const topArtists = ref<ArtistCardData[]>([]);
const artistsLoading = ref(false);
const artistsError = ref(false);

// --- 新碟上架 ---
const newAlbums = ref<AlbumCardData[]>([]);
const albumsLoading = ref(false);
const albumsError = ref(false);

const todayStr = ref('');

const fmCoverUrl = computed(() => getCoverUrl(player.fmSong) || '');
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
  ui.openRoamDrawer();
}

// --- 数据获取 ---
async function fetchRankPlaylists() {
  const cacheKey = 'home_rank';
  const cached = pageCacheGet(cacheKey);
  if (cached) {
    rankPlaylists.value = cached;
    return;
  }
  rankLoading.value = true;
  rankError.value = false;
  try {
    const results = await Promise.allSettled(
      RANK_IDS.map(id => MusicApi.getPlaylistDetail(id))
    );
    rankPlaylists.value = results
      .filter(r => r.status === 'fulfilled')
      .map((r: any) => {
        const data = JSON.parse(r.value);
        return data.playlist;
      })
      .filter(Boolean);
    pageCacheSet(cacheKey, rankPlaylists.value);
  } catch {
    rankError.value = true;
  } finally {
    rankLoading.value = false;
  }
}

async function fetchRecPlaylists() {
  const cacheKey = 'home_rec';
  const cached = pageCacheGet(cacheKey);
  if (cached) {
    recPlaylists.value = cached;
    return;
  }
  recLoading.value = true;
  recError.value = false;
  try {
    // 已登录用 recommend_resource（更精准），未登录用 personalized
    if (userStore.isLoggedIn) {
      const json = await MusicApi.recommendResource();
      const data = JSON.parse(json as string);
      recPlaylists.value = (data.recommend || []).slice(0, 12);
    } else {
      const json = await RecApi.personalized(12);
      const data = JSON.parse(json as string);
      recPlaylists.value = (data.result || []).slice(0, 12);
    }
    pageCacheSet(cacheKey, recPlaylists.value);
  } catch {
    recError.value = true;
  } finally {
    recLoading.value = false;
  }
}

async function fetchNewSongs() {
  const cacheKey = 'home_new_songs';
  const cached = pageCacheGet(cacheKey);
  if (cached) {
    newSongs.value = cached;
    return;
  }
  newSongsLoading.value = true;
  newSongsError.value = false;
  try {
    let list: Song[];
    if (userStore.isLoggedIn) {
      // 已登录：用 personalized_newsong（基于账号口味推荐）
      const json = await RecApi.personalizedNewsong(8);
      const data = JSON.parse(json as string);
      list = (data.result || []).slice(0, 8).map((item: any) => normalizeSong(item.song || item));
    } else {
      // 未登录：用 top_song（全站新歌速递）
      const json = await RecApi.topSong(0);
      const data = JSON.parse(json as string);
      list = (data.data || []).slice(0, 8).map((s: any) => normalizeSong(s));
    }
    newSongs.value = list;
    pageCacheSet(cacheKey, list);
  } catch {
    newSongsError.value = true;
  } finally {
    newSongsLoading.value = false;
  }
}

async function fetchTopArtists() {
  const cacheKey = 'home_top_artists';
  const cached = pageCacheGet(cacheKey);
  if (cached) {
    topArtists.value = cached;
    return;
  }
  artistsLoading.value = true;
  artistsError.value = false;
  try {
    const json = await RecApi.topArtists(8, 0);
    const data = JSON.parse(json as string);
    topArtists.value = (data.artists || []).slice(0, 8);
    pageCacheSet(cacheKey, topArtists.value);
  } catch {
    artistsError.value = true;
  } finally {
    artistsLoading.value = false;
  }
}

async function fetchNewAlbums() {
  const cacheKey = 'home_new_albums';
  const cached = pageCacheGet(cacheKey);
  if (cached) {
    newAlbums.value = cached;
    return;
  }
  albumsLoading.value = true;
  albumsError.value = false;
  try {
    const json = await RecApi.albumNewest();
    const data = JSON.parse(json as string);
    newAlbums.value = (data.albums || []).slice(0, 12);
    pageCacheSet(cacheKey, newAlbums.value);
  } catch {
    albumsError.value = true;
  } finally {
    albumsLoading.value = false;
  }
}

async function loadData() {
  // 并行拉取所有区块（各自带独立缓存）
  await Promise.allSettled([
    fetchRankPlaylists(),
    fetchRecPlaylists(),
    fetchNewSongs(),
    fetchTopArtists(),
    fetchNewAlbums(),
  ]);
}

onMounted(async () => {
  const d = new Date();
  todayStr.value = `${d.getMonth() + 1}月${d.getDate()}日`;
  await loadData();
});

onActivated(() => {
  if (pageCacheIsStale('home_rec') || pageCacheIsStale('home_new_songs')) loadData();
});

// 登录状态变化时清除首页缓存并重新加载（推荐内容依赖登录态）
watch(() => userStore.isLoggedIn, (val, old) => {
  if (val !== old) {
    ['home_rec', 'home_new_songs', 'home_rank', 'home_top_artists', 'home_new_albums']
      .forEach(k => pageCacheInvalidate(k));
    loadData();
  }
});

watch(isOnline, (val, old) => {
  if (val && !old) {
    const hasAny = rankPlaylists.value.length || recPlaylists.value.length
      || newSongs.value.length || topArtists.value.length || newAlbums.value.length;
    if (!hasAny) {
      ['home_rank', 'home_rec', 'home_new_songs', 'home_top_artists', 'home_new_albums']
        .forEach(k => pageCacheInvalidate(k));
      loadData();
    } else {
      if (rankError.value) { pageCacheInvalidate('home_rank'); fetchRankPlaylists(); }
      if (recError.value) { pageCacheInvalidate('home_rec'); fetchRecPlaylists(); }
      if (newSongsError.value) { pageCacheInvalidate('home_new_songs'); fetchNewSongs(); }
      if (artistsError.value) { pageCacheInvalidate('home_top_artists'); fetchTopArtists(); }
      if (albumsError.value) { pageCacheInvalidate('home_new_albums'); fetchNewAlbums(); }
    }
  }
});

// --- 跳转 ---
function goDaily() {
  router.push('/daily');
}

function goPlaylist(id: number) {
  router.push({ name: 'playlist', params: { id } });
}

function goArtist(id: number) {
  router.push({ name: 'artist', params: { id } });
}

function goAlbum(id: number) {
  router.push({ name: 'album', params: { id } });
}

function goLogin() {
  router.push('/login');
}

// --- 播放操作 ---
async function playPlaylist(pl: PlaylistCardData) {
  try {
    // 如果正在播放这个歌单：切换播放/暂停
    if (player.currentSource.type === 'playlist' && player.currentSource.id === pl.id) {
      player.toggle();
      return;
    }
    const json = await MusicApi.playlistTrackAll(pl.id);
    const data = JSON.parse(json as string);
    const songs: Song[] = (data.songs || []).map((s: any) => normalizeSong(s));
    if (songs.length > 0) {
      player.playAll(songs, { type: 'playlist', id: pl.id });
    }
  } catch (e) {
    console.error('播放歌单失败:', e);
  }
}

async function playAlbum(al: AlbumCardData) {
  try {
    // 如果正在播放这个专辑：切换播放/暂停
    if (player.currentSource.type === 'album' && player.currentSource.id === al.id) {
      player.toggle();
      return;
    }
    const json = await MusicApi.albumDetail(al.id);
    const data = JSON.parse(json as string);
    const songs: Song[] = (data.songs || []).map((s: any) => normalizeSong(s));
    if (songs.length > 0) {
      player.playAll(songs, { type: 'album', id: al.id });
    }
  } catch (e) {
    console.error('播放专辑失败:', e);
  }
}

function playNewSong(index: number) {
  if (newSongs.value.length === 0) return;
  player.playFromList(newSongs.value, index);
}

function isCurrentNewSong(song: Song): boolean {
  return player.currentSong?.id === song.id;
}
</script>
