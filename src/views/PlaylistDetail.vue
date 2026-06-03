<template>
  <div class="p-8 text-content">
    <PageHeader />

    <!-- 头部骨架 -->
    <div v-if="!playlist && playlistLoading" class="flex gap-6 mb-8">
      <div class="w-44 h-44 rounded-xl bg-muted animate-pulse flex-shrink-0"></div>
      <div class="flex-1 space-y-3">
        <div class="h-7 bg-muted rounded w-1/2 animate-pulse"></div>
        <div class="h-4 bg-muted rounded w-1/3 animate-pulse"></div>
        <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
        <div class="flex gap-3 mt-4">
          <div class="h-10 w-28 bg-muted rounded-full animate-pulse"></div>
          <div class="h-10 w-20 bg-muted rounded-full animate-pulse"></div>
        </div>
      </div>
    </div>

    <!-- 头部信息 -->
    <div v-else-if="playlist" class="flex gap-6 mb-8">
      <img :src="playlist.coverImgUrl" class="w-44 h-44 rounded-xl object-cover shadow-lg flex-shrink-0" />
      <div class="flex flex-col justify-between min-w-0">
        <div>
          <h1 class="text-2xl font-bold leading-tight">{{ playlist.name }}</h1>
          <div v-if="playlist.creator" class="flex items-center gap-2 mt-2">
            <img :src="playlist.creator.avatarUrl" class="w-5 h-5 rounded-full" />
            <span class="text-sm text-content-2">{{ playlist.creator.nickname }}</span>
          </div>
          <div v-if="playlist.description" class="mt-2">
            <p
              ref="descEl"
              class="text-sm text-content-2 leading-relaxed overflow-hidden"
              style="max-height: 3em"
            >{{ playlist.description }}</p>
            <button
              v-if="descOverflow"
              @click="showDescModal = true"
              class="inline-flex items-center gap-1 text-xs text-accent-text hover:text-accent-text/80 mt-1 px-2 py-0.5 rounded-full bg-accent-text/10 transition"
            >
              <IconChevronDown class="w-3 h-3" />
              查看完整介绍
            </button>
          </div>
          <p class="text-xs text-content-3 mt-2">
            {{ playlist.trackCount }} 首歌曲 · 播放 {{ formatPlayCount(playlist.playCount) }} 次
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
          <button
            v-if="!isOwnPlaylist"
            @click="toggleSubscribe"
            class="px-4 py-2 bg-muted hover:bg-emphasis rounded-full text-sm transition flex items-center gap-2"
            :class="subscribed ? 'text-accent-text' : 'text-content/70'"
          >
            <IconBookmark class="w-4 h-4" :class="subscribed ? 'fill-current' : ''" />
            {{ subscribed ? '已收藏' : '收藏歌单' }}
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
            <h2 class="text-lg font-semibold">{{ playlist?.name }} 的介绍</h2>
            <button @click="showDescModal = false" class="text-content-3 hover:text-content transition">
              <IconX class="w-5 h-5" />
            </button>
          </div>
          <div class="p-5 overflow-y-auto text-sm text-content-2 leading-relaxed whitespace-pre-wrap">{{ playlist?.description }}</div>
        </div>
      </div>
    </Teleport>

    <!-- 加载失败 -->
    <div v-if="loadError" class="flex flex-col items-center justify-center py-16 gap-3">
      <p class="text-content-2 text-sm">加载失败</p>
      <button @click="fetchPlaylist(Number(route.params.id), true)" class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">重试</button>
    </div>

    <!-- 歌曲列表骨架 -->
    <div v-else-if="songsLoading" class="space-y-1">
      <div v-for="i in 8" :key="i" class="flex items-center gap-3 px-3 py-2">
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

    <div v-else-if="!songsLoading && !loadError" class="text-content-2">暂无歌曲</div>

    <div v-if="playlist" class="mt-8">
      <CommentSection :type="2" :id="Number(route.params.id)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onActivated, nextTick } from 'vue';
import { useRoute } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { showToast } from '../composables/useToast';
import { formatPlayCount } from '../utils/format';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet } from '../composables/usePageCache';
import SongListItem from '../components/SongListItem.vue';
import CommentSection from '../components/CommentSection.vue';
import PageHeader from '../components/PageHeader.vue';
import IconPlay from '~icons/lucide/play';
import IconBookmark from '~icons/lucide/bookmark';
import IconX from '~icons/lucide/x';
import IconChevronDown from '~icons/lucide/chevron-down';

defineOptions({ name: 'PlaylistDetailView' });

const route = useRoute();
const player = usePlayerStore();
const userStore = useUserStore();

const playlist = ref<any>(null);
const songs = ref<Song[]>([]);
const playlistLoading = ref(true);
const songsLoading = ref(false);
const loadError = ref(false);
const subscribed = ref(false);
const showDescModal = ref(false);
const descOverflow = ref(false);
const descEl = ref<HTMLElement | null>(null);

const isOwnPlaylist = computed(() => {
  if (!playlist.value || !userStore.user) return false;
  return playlist.value.creator?.userId === userStore.user.userId;
});

function checkDescOverflow() {
  nextTick(() => {
    if (descEl.value) {
      descOverflow.value = descEl.value.scrollHeight > descEl.value.clientHeight + 2;
    }
  });
}

async function fetchPlaylist(id: number, force = false) {
  const cacheKey = `playlist_${id}`;
  if (!force) {
    const cached = pageCacheGet(cacheKey);
    if (cached) {
      playlist.value = cached.playlist;
      songs.value = cached.songs;
      subscribed.value = cached.subscribed;
      playlistLoading.value = false;
      songsLoading.value = false;
      loadError.value = false;
      checkDescOverflow();
      return;
    }
  }

  playlistLoading.value = true;
  songsLoading.value = true;
  loadError.value = false;
  playlist.value = null;
  songs.value = [];
  try {
    const jsonStr: string = await MusicApi.getPlaylistDetail(id);
    const data = JSON.parse(jsonStr);
    playlist.value = data.playlist;
    playlistLoading.value = false;
    songs.value = (data.playlist.tracks || []).map(normalizeSong);
    songsLoading.value = false;
    subscribed.value = data.playlist.subscribed || false;
    pageCacheSet(cacheKey, { playlist: playlist.value, songs: songs.value, subscribed: subscribed.value });
    checkDescOverflow();
  } catch (e) {
    console.error(e);
    loadError.value = true;
    playlistLoading.value = false;
    songsLoading.value = false;
    showToast('获取歌单详情失败', 'error');
  }
}

onMounted(() => {
  fetchPlaylist(Number(route.params.id));
});

watch(() => route.params.id, (newId) => {
  if (newId && route.name === 'playlist') fetchPlaylist(Number(newId));
});

onActivated(() => {
  if (loadError.value) fetchPlaylist(Number(route.params.id), true);
});

function playAll() {
  if (songs.value.length === 0) return;
  player.playAll(songs.value);
}

async function toggleSubscribe() {
  if (!playlist.value) return;
  const newSubscribed = !subscribed.value;
  try {
    await MusicApi.playlistSubscribe(Number(playlist.value.id), newSubscribed);
    subscribed.value = newSubscribed;
    showToast(subscribed.value ? '已收藏歌单' : '已取消收藏', 'success');
  } catch {
    showToast('操作失败，请稍后重试', 'error');
  }
}
</script>
