<template>
  <DetailLayout ref="layoutRef">
    <!-- 头部（常驻）：滚动时收缩为紧凑态
         参考 SPlayer ListDetail.vue .small 触发 height/font-size/opacity 过渡 -->
    <template #header="{ compact }">
      <div class="album-header" :class="{ 'is-compact': compact }">
        <!-- 头部骨架 -->
        <div v-if="!album && albumLoading" class="header-row">
          <div class="cover-wrap">
            <div class="cover-img cover-skeleton"></div>
          </div>
          <div class="info-col">
            <div class="h-8 bg-muted rounded w-1/2 animate-pulse"></div>
            <div class="info-collapse">
              <div class="h-4 bg-muted rounded w-1/3 animate-pulse mt-3"></div>
              <div class="h-3 bg-muted rounded w-1/4 animate-pulse mt-2"></div>
            </div>
            <div class="actions-row">
              <div class="h-10 w-28 bg-muted rounded-full animate-pulse"></div>
            </div>
          </div>
        </div>

        <!-- 头部信息 -->
        <div v-else-if="album" class="header-row">
          <!-- 封面 + 阴影背板 + 顶部渐变遮罩（参考 SPlayer .cover + .cover-shadow + .cover-mask）
               封面图加载完成后淡入（opacity 0→1），避免空白闪烁 -->
          <div class="cover-wrap">
            <img :src="album.picUrl" class="cover-shadow" alt="" aria-hidden="true" />
            <img
              :src="album.picUrl"
              class="cover-img"
              :class="{ 'is-loaded': coverLoaded }"
              :alt="album.name"
              @load="coverLoaded = true"
            />
            <!-- 顶部渐变遮罩：紧凑时淡出（参考 SPlayer .cover-mask） -->
            <div class="cover-mask"></div>
          </div>

          <!-- 信息列：标题（常驻）+ 元信息（紧凑时折叠）+ 操作行（常驻底部）
               操作行内左侧播放按钮，右侧模糊搜索 + 歌曲/评论 tab（参考 SPlayer .menu .left/.right） -->
          <div class="info-col">
            <div class="info-top min-w-0">
              <h1 class="album-title font-bold leading-tight">{{ album.name }}</h1>

              <!-- 元信息：紧凑时通过 max-height + opacity 折叠（参考 SPlayer n-collapse-transition） -->
              <div class="info-collapse">
                <!-- 艺术家行 -->
                <div v-if="album.artists?.length" class="meta-row">
                  <IconUserRound class="meta-icon" />
                  <div class="flex flex-wrap items-center gap-x-1 gap-y-0.5">
                    <template v-for="(ar, idx) in album.artists" :key="ar.id">
                      <span v-if="(idx as number) > 0" class="text-content-3">/</span>
                      <span
                        class="meta-text hover:text-accent-text cursor-pointer transition whitespace-nowrap"
                        @click="ar.id && router.push({ name: 'artist', params: { id: ar.id } })"
                      >{{ ar.name }}</span>
                    </template>
                  </div>
                </div>

                <!-- 歌曲数 + 发行日期 -->
                <div class="meta-row">
                  <IconMusic class="meta-icon" />
                  <span class="meta-text">{{ songs.length }} 首歌曲</span>
                  <IconCalendar class="meta-icon ml-3" v-if="album.publishTime" />
                  <span class="meta-text" v-if="album.publishTime">{{ formatDate(album.publishTime) }}</span>
                </div>

                <!-- 简介（如有） -->
                <p v-if="album.description" class="album-desc meta-text mt-2">
                  {{ album.description }}
                </p>
              </div>
            </div>

            <!-- 操作行：始终在信息列底部（参考 SPlayer .menu position:absolute bottom:0）
                 左侧播放按钮常驻；右侧模糊搜索 + 歌曲/评论 tab（窄屏隐藏） -->
            <div class="actions-row">
              <div class="actions-left">
                <button
                  @click="playAll"
                  class="play-btn bg-accent hover:bg-accent-hover text-white font-medium rounded-full transition flex items-center gap-2"
                >
                  <IconPlay class="w-4 h-4 fill-current" />
                  <span>播放全部</span>
                </button>
              </div>
              <div class="actions-right">
                <!-- 模糊搜索：过滤当前歌曲列表 -->
                <div class="search-wrap" @click.stop>
                  <IconSearch class="search-icon" />
                  <input
                    v-model="searchKeyword"
                    placeholder="模糊搜索"
                    class="search-input"
                  />
                  <button
                    v-if="searchKeyword"
                    @click="searchKeyword = ''"
                    class="search-clear"
                  >
                    <IconX class="w-3 h-3" />
                  </button>
                </div>
                <!-- 歌曲 / 评论 tab -->
                <div class="tabs-wrap">
                  <button
                    class="tab-btn"
                    :class="{ active: currentTab === 'songs' }"
                    @click="currentTab = 'songs'"
                  >歌曲<span class="tab-count">{{ songs.length }}</span></button>
                  <button
                    class="tab-btn"
                    :class="{ active: currentTab === 'comments' }"
                    @click="currentTab = 'comments'"
                  >评论</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 内容区（独立滚动）：歌曲 / 评论 按 tab 切换渲染
         songs 用 v-if 仅激活时渲染；comments 用「首次激活后保活」模式（v-if + v-show）保留滚动位置 -->
    <template #content>
      <div class="album-list-inner">
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

        <!-- 歌曲 tab：v-if 仅在激活时渲染 -->
        <template v-else-if="currentTab === 'songs'">
          <VirtualSongList
            v-if="filteredSongs.length"
            :songs="filteredSongs"
            :current-song-id="player.currentSong?.id"
            show-index
            show-like
            show-download
            show-menu
            show-duration
            show-playing-overlay
            @song-click="(_s, i) => player.playFromList(filteredSongs, i)"
          />
          <!-- 搜索无结果 -->
          <div v-else-if="searchKeyword" class="flex flex-col items-center justify-center py-16 gap-2 text-content-3 text-sm">
            <p>搜不到关于 {{ searchKeyword }} 的任何歌曲</p>
          </div>
          <!-- 空状态 -->
          <div v-else class="flex flex-col items-center justify-center py-16 gap-2 text-content-3 text-sm">
            <p>暂无歌曲</p>
          </div>
        </template>

        <!-- 评论 tab：首次切到时挂载（commentsActivated=true），之后保活以保留滚动位置 -->
        <CommentSection
          v-if="commentsActivated"
          v-show="currentTab === 'comments'"
          :type="3"
          :id="Number(route.params.id)"
        />
      </div>
    </template>
  </DetailLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onActivated } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { normalizeSong, type Song } from '../utils/song';
import { formatDate } from '../utils/format';
import { pageCacheGet, pageCacheSet } from '../composables/usePageCache';
import VirtualSongList from '../components/VirtualSongList.vue';
import CommentSection from '../components/CommentSection.vue';
import DetailLayout from '../components/DetailLayout.vue';
import IconPlay from '~icons/lucide/play';
import IconUserRound from '~icons/lucide/user-round';
import IconMusic from '~icons/lucide/music';
import IconCalendar from '~icons/lucide/calendar';
import IconSearch from '~icons/lucide/search';
import IconX from '~icons/lucide/x';

defineOptions({ name: 'AlbumDetailView' });

const route = useRoute();
const router = useRouter();
const player = usePlayerStore();

const album = ref<any>(null);
const songs = ref<Song[]>([]);
const albumLoading = ref(true);
const songsLoading = ref(false);
const loadError = ref(false);

// 封面加载完成标记（控制淡入）
const coverLoaded = ref(false);

// 当前 tab：songs / comments（参考 SPlayer currentTab）
const currentTab = ref<'songs' | 'comments'>('songs');
// 评论 tab 首次激活标记：懒加载 + 保活（v-if 控制挂载，v-show 控制显示）
const commentsActivated = ref(false);

// 模糊搜索关键词（参考 SPlayer searchValue）
const searchKeyword = ref('');

// DetailLayout 实例引用：用于切换专辑/tab 时重置滚动 & 紧凑态
const layoutRef = ref<InstanceType<typeof DetailLayout> | null>(null);

// 过滤后的歌曲列表：按关键词模糊匹配歌名/歌手/专辑名
const filteredSongs = computed<Song[]>(() => {
  const kw = searchKeyword.value.trim().toLowerCase();
  if (!kw) return songs.value;
  return songs.value.filter((s) => {
    const name = (s.name || '').toLowerCase();
    const artist = (s.ar?.map((a: any) => a.name).join('') || '').toLowerCase();
    const albumName = (s.al?.name || '').toLowerCase();
    return name.includes(kw) || artist.includes(kw) || albumName.includes(kw);
  });
});

// 切换 tab 时重置紧凑态：评论/歌曲高度差异大，避免紧凑态卡死
// 首次切到评论 tab 时激活保活标记
watch(currentTab, (tab) => {
  layoutRef.value?.resetScroll();
  if (tab === 'comments') commentsActivated.value = true;
});

async function fetchAlbum(id: number, force = false) {
  const cacheKey = `album_${id}`;
  // 切换专辑时重置 tab、搜索、封面与滚动状态（参考 SPlayer watch detailData.id）
  currentTab.value = 'songs';
  commentsActivated.value = false;
  searchKeyword.value = '';
  coverLoaded.value = false;
  layoutRef.value?.resetScroll();
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
    console.error('获取专辑详情失败', e);
    loadError.value = true;
    albumLoading.value = false;
    songsLoading.value = false;
  }
}

onMounted(() => {
  fetchAlbum(Number(route.params.id));
});

// 合并两个 route.params.id watcher：仅监听一次，同时处理状态重置和数据获取
watch(() => route.params.id, (newId) => {
  if (newId && route.name === 'album') fetchAlbum(Number(newId));
});

onActivated(() => {
  if (loadError.value) fetchAlbum(Number(route.params.id), true);
});

function playAll() {
  // 播放当前过滤后的列表（搜索时只播放匹配项）
  const list = filteredSongs.value.length ? filteredSongs.value : songs.value;
  if (list.length === 0) return;
  player.playAll(list);
}
</script>

<style scoped>
/* === 头部容器：固定高度 + padding，紧凑态收窄 ===
   参考 SPlayer .detail height 240→120 过渡。
   根容器布局由 DetailLayout 提供（absolute + flex 列 + header flex-shrink:0）。 */
.album-header {
  height: 240px;
  z-index: 1;
  padding: 12px 32px 24px 32px;
  border-bottom: 1px solid transparent;
  transition:
    height 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    border-color 0.3s ease;
}
.album-header.is-compact {
  height: 120px;
  padding: 12px 32px 12px 32px;
  border-bottom-color: var(--c-line-2);
}

/* 头部内行：封面 + 信息列横向排列 */
.header-row {
  display: flex;
  gap: 24px;
  height: 100%;
  align-items: stretch;
}
.album-header.is-compact .header-row {
  align-items: center;
}

/* === 封面区（参考 SPlayer .cover + .cover-shadow） ===
   正方形，高度跟随 header 高度（240/120），宽度等于高度。
   position:relative 承载阴影背板。 */
.cover-wrap {
  position: relative;
  height: 100%;
  aspect-ratio: 1 / 1;
  flex-shrink: 0;
  border-radius: 8px;
  transition: height 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 主封面图：加载完成后淡入（参考 SPlayer img opacity 0→1 + .is-loaded）
   未加载时 opacity:0 避免空白闪烁；加载后 .is-loaded 触发淡入 */
.cover-img {
  position: relative;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 8px;
  z-index: 1;
  opacity: 0;
  transition: opacity 0.35s ease-in-out;
}
.cover-img.is-loaded {
  opacity: 1;
}
/* 骨架占位 */
.cover-skeleton {
  background-color: var(--c-muted);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

/* 阴影背板：模糊 + 半透明 + 略小，营造封面浮起的氛围 */
.cover-shadow {
  position: absolute;
  top: 6px;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 8px;
  filter: blur(12px) opacity(0.6);
  transform: scale(0.92, 0.96);
  z-index: 0;
  pointer-events: none;
}

/* 顶部渐变遮罩（参考 SPlayer .cover-mask）：紧凑时淡出 */
.cover-mask {
  position: absolute;
  top: 0;
  left: 0;
  height: 30%;
  width: 100%;
  border-radius: 8px;
  overflow: hidden;
  z-index: 2;
  background: linear-gradient(rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0));
  transition: opacity 0.3s ease;
  pointer-events: none;
}
.album-header.is-compact .cover-mask {
  opacity: 0;
}

/* === 信息列（参考 SPlayer .data） ===
   flex:1 占满剩余宽度，纵向 flex：顶部信息 + 底部操作行。
   justify-content:space-between 让操作行始终贴底。 */
.info-col {
  position: relative;
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding-right: 60px;
}

/* === 标题（参考 SPlayer .name） ===
   展开 30px，紧凑 22px。固定 line-height + min-height 防字号变化抖动。 */
.album-title {
  font-size: 30px;
  margin-bottom: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.3;
  min-height: 39px;
  transition: font-size 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.album-header.is-compact .album-title {
  font-size: 22px;
  margin-bottom: 0;
}

/* === 元信息折叠区（参考 SPlayer n-collapse-transition） ===
   展开 max-height 大、opacity 1；紧凑 max-height 0、opacity 0、收起间距。 */
.info-collapse {
  max-height: 160px;
  opacity: 1;
  overflow: hidden;
  transition:
    max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.2s ease,
    margin 0.3s ease;
}
.album-header.is-compact .info-collapse {
  max-height: 0;
  opacity: 0;
  margin-top: -4px;
  pointer-events: none;
}

/* 元信息行：图标 + 文字，多行堆叠 */
.meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  font-size: 14px;
}
.meta-row:first-of-type {
  margin-top: 0;
}
.meta-icon {
  width: 18px;
  height: 18px;
  color: var(--c-content-3);
  flex-shrink: 0;
}
.meta-text {
  color: var(--c-content-2);
  font-size: 14px;
}

/* 简介：单行省略 */
.album-desc {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

/* === 操作行（参考 SPlayer .menu position:absolute bottom:0） ===
   始终在信息列底部；左侧播放按钮 + 右侧搜索/Tab，两端对齐。 */
.actions-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.actions-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.actions-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}
.play-btn {
  height: 40px;
  padding: 0 20px;
  font-size: 14px;
  transition:
    height 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    background-color 0.2s ease;
}
.album-header.is-compact .play-btn {
  height: 32px;
  padding: 0 16px;
  font-size: 13px;
}

/* 模糊搜索框（参考 SPlayer .search）：胶囊形，聚焦时变宽 */
.search-wrap {
  position: relative;
  display: flex;
  align-items: center;
  height: 40px;
  width: 130px;
  border-radius: 25px;
  background-color: var(--c-subtle);
  border: 1px solid var(--c-line-2);
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1), height 0.3s ease;
}
.search-wrap:focus-within {
  width: 200px;
  border-color: var(--c-accent);
}
.search-icon {
  width: 16px;
  height: 16px;
  margin-left: 12px;
  color: var(--c-content-3);
  flex-shrink: 0;
}
.search-input {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0 8px;
  background: transparent;
  border: none;
  outline: none;
  font-size: 13px;
  color: var(--c-content);
}
.search-input::placeholder { color: var(--c-content-3); }
.search-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 8px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  color: var(--c-content-3);
  flex-shrink: 0;
  transition: background-color 0.2s, color 0.2s;
}
.search-clear:hover {
  background-color: var(--c-muted);
  color: var(--c-content);
}

/* 歌曲/评论 tab（参考 SPlayer .tabs type=segment） */
.tabs-wrap {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 3px;
  border-radius: 25px;
  background-color: var(--c-subtle);
  border: 1px solid var(--c-line-2);
  transition: height 0.3s ease;
}
.tab-btn {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  padding: 0 14px;
  border-radius: 22px;
  font-size: 13px;
  color: var(--c-content-2);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background-color 0.3s, color 0.3s;
}
.tab-btn.active {
  background-color: var(--c-accent);
  color: #fff;
}
.tab-btn:not(.active):hover {
  color: var(--c-content);
}
.tab-count {
  font-size: 11px;
  opacity: 0.7;
  transform: translateY(-4px);
}

/* 紧凑态：右侧搜索/tab 高度缩小（参考 SPlayer .small .menu） */
.album-header.is-compact .search-wrap,
.album-header.is-compact .tabs-wrap {
  height: 32px;
}
.album-header.is-compact .search-wrap {
  width: 110px;
}
.album-header.is-compact .search-wrap:focus-within {
  width: 160px;
}
.album-header.is-compact .tab-btn {
  padding: 0 10px;
  font-size: 12px;
}

/* 窄屏隐藏右侧搜索/tab（参考 SPlayer @media max-width:1200px .right） */
@media (max-width: 1200px) {
  .actions-right {
    display: none;
  }
}

/* 窄屏适配（参考 SPlayer @media） */
@media (max-width: 768px) {
  .album-header {
    height: 180px;
    padding: 12px 16px 20px 16px;
  }
  .album-header.is-compact {
    height: 100px;
  }
  .header-row {
    gap: 12px;
  }
  .info-col {
    padding-right: 12px;
  }
  .album-title {
    font-size: 22px;
  }
  .album-header.is-compact .album-title {
    font-size: 18px;
  }
}

/* 列表内层 padding（外层滚动容器由 DetailLayout 提供） */
.album-list-inner {
  padding: 0 32px 32px 32px;
}
@media (max-width: 768px) {
  .album-list-inner {
    padding: 0 16px 24px 16px;
  }
}
</style>
