<template>
  <DetailLayout ref="layoutRef">
    <!-- 头部（常驻）：滚动时收缩为紧凑态（封面缩小、标题缩小、介绍隐藏） -->
    <template #header="{ compact }">
      <div class="pl-header" :class="{ 'is-compact': compact }">
        <PageHeader />

        <!-- 头部骨架 -->
        <div v-if="!playlist && playlistLoading" class="pl-head-row">
          <div class="pl-cover cover-skeleton"></div>
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
        <div v-else-if="playlist" class="pl-head-row">
          <img :src="playlist.coverImgUrl" class="pl-cover" />
          <div class="pl-info">
            <div class="pl-info-top">
              <h1 class="pl-title">{{ playlist.name }}</h1>
              <!-- 元信息（紧凑时折叠隐藏，参考 SPlayer n-collapse-transition） -->
              <div class="pl-collapse">
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
            </div>

            <!-- 操作行：播放按钮 + tab（始终常驻底部） -->
            <div class="pl-actions">
              <button
                @click="playAll"
                class="pl-play-btn"
              >
                <IconPlay class="w-4 h-4 fill-current" />
                <span>播放全部</span>
              </button>
              <button
                v-if="!isOwnPlaylist"
                @click="toggleSubscribe"
                class="pl-sub-btn"
                :class="subscribed ? 'text-accent-text' : 'text-content/70'"
              >
                <IconBookmark class="w-4 h-4" :class="subscribed ? 'fill-current' : ''" />
                {{ subscribed ? '已收藏' : '收藏歌单' }}
              </button>

              <!-- 歌曲 / 评论 tab（紧凑态缩小） -->
              <div class="pl-tabs">
                <button
                  class="pl-tab"
                  :class="{ active: currentTab === 'songs' }"
                  @click="currentTab = 'songs'"
                >歌曲<span class="pl-tab-count">{{ songs.length }}</span></button>
                <button
                  class="pl-tab"
                  :class="{ active: currentTab === 'comments' }"
                  @click="currentTab = 'comments'"
                >评论</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 内容区（独立滚动）：歌曲 / 评论 按 tab 切换渲染 -->
    <template #content>
      <div class="px-8 pb-8 text-content">
        <!-- 加载失败 -->
        <div v-if="loadError" class="flex flex-col items-center justify-center py-16 gap-3">
          <p class="text-content-2 text-sm">加载失败</p>
          <button @click="fetchPlaylist(Number(route.params.id), true)" class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">重试</button>
        </div>

        <template v-else>
          <!-- 歌曲 tab：v-if 仅在激活时渲染（参考 SPlayer） -->
          <div v-if="currentTab === 'songs'">
            <!-- 歌曲列表骨架 -->
            <div v-if="songsLoading" class="space-y-1">
              <div v-for="i in 8" :key="i" class="flex items-center gap-3 px-3 py-2">
                <div class="w-12 h-12 bg-muted rounded animate-pulse flex-shrink-0"></div>
                <div class="flex-1 space-y-2">
                  <div class="h-4 bg-muted rounded w-2/3 animate-pulse"></div>
                  <div class="h-3 bg-muted rounded w-1/3 animate-pulse"></div>
                </div>
              </div>
            </div>
            <!-- 歌曲列表 -->
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
            <div v-else-if="!songsLoading" class="text-content-2">暂无歌曲</div>
          </div>

          <!-- 评论 tab：首次切到时挂载（commentsActivated=true），之后保活以保留滚动位置 -->
          <CommentSection
            v-if="commentsActivated && playlist"
            v-show="currentTab === 'comments'"
            :type="2"
            :id="Number(route.params.id)"
          />
        </template>
      </div>
    </template>
  </DetailLayout>

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
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onActivated } from 'vue';
import { useRoute } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { useUserStore } from '../stores/user';
import { showToast } from '../composables/useToast';
import { formatPlayCount } from '../utils/format';
import { checkOverflow } from '../utils/dom';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet } from '../composables/usePageCache';
import VirtualSongList from '../components/VirtualSongList.vue';
import CommentSection from '../components/CommentSection.vue';
import PageHeader from '../components/PageHeader.vue';
import DetailLayout from '../components/DetailLayout.vue';
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
const layoutRef = ref<InstanceType<typeof DetailLayout> | null>(null);
const currentTab = ref<'songs' | 'comments'>('songs');
// 评论 tab 首次激活标记：懒加载 + 保活（v-if 控制挂载，v-show 控制显示）
const commentsActivated = ref(false);

const isOwnPlaylist = computed(() => {
  if (!playlist.value || !userStore.user) return false;
  return playlist.value.creator?.userId === userStore.user.userId;
});

function checkDescOverflow() {
  checkOverflow(descEl, descOverflow);
}

async function fetchPlaylist(id: number, force = false) {
  const cacheKey = `playlist_${id}`;
  // 切换歌单时重置 tab、评论激活与滚动状态
  currentTab.value = 'songs';
  commentsActivated.value = false;
  layoutRef.value?.resetScroll();
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
    console.error('获取歌单详情失败', e);
    loadError.value = true;
    playlistLoading.value = false;
    songsLoading.value = false;
    showToast('获取歌单详情失败', 'error');
  }
}

// 切换 tab 时重置紧凑态（评论/歌曲高度差异大，避免紧凑态卡死）
// 首次切到评论 tab 时激活保活标记
watch(currentTab, (tab) => {
  layoutRef.value?.resetScroll();
  if (tab === 'comments') commentsActivated.value = true;
});

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

<style scoped>
/* 头部容器：固定 padding，紧凑态收窄 padding 让出空间给列表
   参考 SPlayer .detail height 240→120 过渡 */
.pl-header {
  padding: 12px 32px 24px 32px;
  transition: padding 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.pl-header.is-compact {
  padding: 12px 32px 12px 32px;
}

/* 头部行：封面 + 信息横向排列 */
.pl-head-row {
  display: flex;
  gap: 24px;
  margin-top: 16px;
}

/* 封面：176→120 紧凑过渡（参考 SPlayer .cover height:100% of 240/120） */
.pl-cover {
  width: 176px;
  height: 176px;
  border-radius: 12px;
  object-fit: cover;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
  transition:
    width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    height 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.pl-cover.cover-skeleton {
  background-color: var(--c-muted);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
.pl-header.is-compact .pl-cover {
  width: 96px;
  height: 96px;
}

/* 信息列：纵向占满，顶部标题+折叠区，底部操作行 */
.pl-info {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-width: 0;
  flex: 1;
}
.pl-info-top {
  min-width: 0;
}

/* 标题：24→18 紧凑过渡（参考 SPlayer .name 30→22） */
.pl-title {
  font-size: 24px;
  font-weight: bold;
  line-height: 1.3;
  transition: font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.pl-header.is-compact .pl-title {
  font-size: 18px;
}

/* 元信息折叠区：紧凑时 max-height:0 + opacity:0 隐藏（参考 SPlayer n-collapse-transition） */
.pl-collapse {
  max-height: 200px;
  opacity: 1;
  overflow: hidden;
  transition:
    max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.3s ease;
}
.pl-header.is-compact .pl-collapse {
  max-height: 0;
  opacity: 0;
}

/* 操作行：播放按钮 + 收藏 + tab，紧凑态缩小 */
.pl-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
}
.pl-play-btn {
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
.pl-play-btn:hover {
  background-color: var(--c-accent-hover);
}
.pl-sub-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background-color: var(--c-muted);
  border-radius: 9999px;
  font-size: 14px;
  transition:
    background-color 0.2s ease,
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.pl-sub-btn:hover {
  background-color: var(--c-emphasis);
}
.pl-header.is-compact .pl-play-btn,
.pl-header.is-compact .pl-sub-btn {
  padding: 6px 14px;
  font-size: 13px;
}

/* 歌曲/评论 tab：胶囊形，紧凑态缩小 */
.pl-tabs {
  display: flex;
  gap: 4px;
  margin-left: auto;
  background-color: var(--c-subtle);
  padding: 3px;
  border-radius: 9999px;
  transition: padding 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.pl-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 14px;
  border-radius: 9999px;
  font-size: 14px;
  color: var(--c-content-2);
  transition:
    background-color 0.2s ease,
    color 0.2s ease,
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.pl-tab.active {
  background-color: var(--c-accent);
  color: #fff;
}
.pl-tab-count {
  font-size: 12px;
  opacity: 0.7;
}
.pl-header.is-compact .pl-tab {
  padding: 4px 10px;
  font-size: 13px;
}
</style>
