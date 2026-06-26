<template>
  <DetailLayout ref="layoutRef">
    <!-- 头部（常驻）：滚动时收缩为紧凑态（头像缩小、标题缩小、简介隐藏）
         参考 SPlayer ListDetail.vue .small 触发 height/font-size/opacity 过渡 -->
    <template #header="{ compact }">
      <div class="ar-header" :class="{ 'is-compact': compact }">
        <PageHeader />

        <!-- 头部骨架 -->
        <div v-if="!artist && !songs.length && !albums.length" class="ar-head-row">
          <div class="ar-cover cover-skeleton"></div>
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

        <template v-if="!loadError && (artist || songs.length || albums.length)">
          <!-- 头部行：头像 + 信息（标题/简介/操作按钮 + tab） -->
          <div class="ar-head-row">
            <img v-if="artistCover" :src="artistCover" class="ar-cover" />
            <div v-else class="ar-cover ar-cover-placeholder">
              <IconMusic class="w-12 h-12 text-content-4" />
            </div>
            <div class="ar-info">
              <div class="ar-info-top">
                <h1 class="ar-title">{{ artistName }}</h1>
                <!-- 元信息（紧凑时折叠隐藏，参考 SPlayer n-collapse-transition） -->
                <div class="ar-collapse">
                  <p v-if="artistFollowers || artist?.musicSize" class="text-xs text-content-3 mt-1">
                    <span v-if="artistFollowers">{{ formatPlayCount(artistFollowers) }} 粉丝</span>
                    <span v-if="artistFollowers && artist?.musicSize"> · </span>
                    <span v-if="artist?.musicSize">{{ artist.musicSize }} 首歌曲</span>
                  </p>
                  <div v-if="briefDesc" class="mt-2">
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
                </div>
              </div>

              <!-- 操作行：播放按钮 + 关注按钮 + tab（始终常驻底部） -->
              <div class="ar-actions">
                <button
                  @click="playAll"
                  class="ar-play-btn"
                >
                  <IconPlay class="w-4 h-4 fill-current" />
                  <span>播放全部</span>
                </button>
                <button
                  @click="toggleFollow"
                  :disabled="followLoading"
                  class="ar-follow-btn"
                  :class="{ 'is-followed': isFollowed }"
                >
                  {{ isFollowed ? '已关注' : '关注' }}
                </button>

                <!-- 热门歌曲 / 专辑 tab（紧凑态缩小） -->
                <div class="ar-tabs">
                  <button
                    v-for="tab in tabs"
                    :key="tab.key"
                    class="ar-tab"
                    :class="{ active: activeTab === tab.key }"
                    @click="activeTab = tab.key"
                  >{{ tab.label }}</button>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </template>

    <!-- 内容区（独立滚动）：歌曲列表 / 专辑列表 按 tab 切换渲染 -->
    <template #content>
      <div class="px-8 pb-8 text-content" v-if="!loadError">
        <!-- 歌曲 tab：v-if 仅在激活时渲染（无评论需保活，两 tab 均 v-if） -->
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

        <!-- 专辑 tab：v-if 仅在激活时渲染 -->
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
      </div>
    </template>
  </DetailLayout>

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
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onActivated } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { formatPlayCount, formatDate } from '../utils/format';
import { checkOverflow } from '../utils/dom';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet } from '../composables/usePageCache';
import VirtualSongList from '../components/VirtualSongList.vue';
import PageHeader from '../components/PageHeader.vue';
import DetailLayout from '../components/DetailLayout.vue';
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
const activeTab = ref<'songs' | 'albums'>('songs');
const showDescModal = ref(false);
const descOverflow = ref(false);
const descEl = ref<HTMLElement | null>(null);
const isFollowed = ref(false);
const followLoading = ref(false);
// DetailLayout 实例引用：用于切换歌手时重置滚动 & 紧凑态
const layoutRef = ref<InstanceType<typeof DetailLayout> | null>(null);

const tabs = [
  { key: 'songs' as const, label: '热门歌曲' },
  { key: 'albums' as const, label: '专辑' },
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
  checkOverflow(descEl, descOverflow);
}

async function fetchArtist(id: number, force = false) {
  const cacheKey = `artist_${id}`;
  // 切换歌手时重置 tab 与滚动状态（参考 SPlayer watch detailData.id）
  activeTab.value = 'songs';
  layoutRef.value?.resetScroll();
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

<style scoped>
/* 头部容器：固定 padding，紧凑态收窄 padding 让出空间给列表
   参考 SPlayer .detail height 240→120 过渡 */
.ar-header {
  padding: 12px 32px 24px 32px;
  transition: padding 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-header.is-compact {
  padding: 12px 32px 12px 32px;
}

/* 头部行：头像 + 信息横向排列 */
.ar-head-row {
  display: flex;
  gap: 24px;
  margin-top: 16px;
}

/* 头像：圆形，176→96 紧凑过渡（参考 SPlayer .cover height:100% of 240/120） */
.ar-cover {
  width: 176px;
  height: 176px;
  border-radius: 50%;
  object-fit: cover;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
  transition:
    width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    height 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-cover.cover-skeleton {
  background-color: var(--c-muted);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
.ar-cover-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--c-muted);
}
.ar-header.is-compact .ar-cover {
  width: 96px;
  height: 96px;
}

/* 信息列：纵向占满，顶部标题+折叠区，底部操作行 */
.ar-info {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-width: 0;
  flex: 1;
}
.ar-info-top {
  min-width: 0;
}

/* 标题：24→18 紧凑过渡（参考 SPlayer .name 30→22）
   固定 line-height + min-height 防止字号变化时抖动 */
.ar-title {
  font-size: 24px;
  font-weight: bold;
  line-height: 1.3;
  min-height: 32px;
  transition: font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-header.is-compact .ar-title {
  font-size: 18px;
}

/* 元信息折叠区：紧凑时 max-height:0 + opacity:0 隐藏（参考 SPlayer n-collapse-transition） */
.ar-collapse {
  max-height: 200px;
  opacity: 1;
  overflow: hidden;
  transition:
    max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.3s ease;
}
.ar-header.is-compact .ar-collapse {
  max-height: 0;
  opacity: 0;
}

/* 操作行：播放按钮 + 关注 + tab，紧凑态缩小 */
.ar-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
}
.ar-play-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  background-color: var(--c-accent);
  color: #fff;
  font-weight: 500;
  border-radius: 9999px;
  transition:
    background-color 0.2s ease,
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-play-btn:hover {
  background-color: var(--c-accent-hover);
}
.ar-follow-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background-color: var(--c-accent-dim);
  color: var(--c-accent-text);
  border-radius: 9999px;
  font-size: 14px;
  transition:
    background-color 0.2s ease,
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-follow-btn.is-followed {
  background-color: var(--c-subtle);
  color: var(--c-content-2);
}
.ar-follow-btn:not(.is-followed):hover {
  background-color: var(--c-emphasis);
}
.ar-follow-btn.is-followed:hover {
  background-color: var(--c-muted);
}
.ar-header.is-compact .ar-play-btn,
.ar-header.is-compact .ar-follow-btn {
  padding: 6px 14px;
  font-size: 13px;
}

/* 热门歌曲/专辑 tab：胶囊形，紧凑态缩小 */
.ar-tabs {
  display: flex;
  gap: 4px;
  margin-left: auto;
  background-color: var(--c-subtle);
  padding: 3px;
  border-radius: 9999px;
  transition: padding 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-tab {
  display: flex;
  align-items: center;
  padding: 6px 14px;
  font-size: 13px;
  color: var(--c-content-2);
  background: transparent;
  border: none;
  border-radius: 9999px;
  cursor: pointer;
  transition:
    background-color 0.2s ease,
    color 0.2s ease,
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.ar-tab.active {
  background-color: var(--c-accent);
  color: #fff;
}
.ar-tab:not(.active):hover {
  color: var(--c-content);
}
.ar-header.is-compact .ar-tab {
  padding: 5px 10px;
  font-size: 12px;
}

/* 窄屏适配（参考 SPlayer @media） */
@media (max-width: 768px) {
  .ar-header {
    padding: 12px 16px 20px 16px;
  }
  .ar-header.is-compact {
    padding: 12px 16px 12px 16px;
  }
  .ar-head-row {
    gap: 12px;
  }
  .ar-cover {
    width: 120px;
    height: 120px;
  }
  .ar-header.is-compact .ar-cover {
    width: 72px;
    height: 72px;
  }
  .ar-title {
    font-size: 20px;
  }
  .ar-header.is-compact .ar-title {
    font-size: 16px;
  }
  /* 窄屏隐藏 tab 文字以节省空间（仅活跃 tab 显示） */
  .ar-tabs {
    gap: 2px;
    padding: 2px;
  }
  .ar-tab {
    padding: 5px 10px;
    font-size: 12px;
  }
}
</style>
