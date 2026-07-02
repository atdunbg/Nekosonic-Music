<template>
  <div class="p-8 text-content">
    <div v-if="!hasSearched && !loading && hotTags.length" class="mb-6">
      <h2 class="text-sm font-semibold mb-3">🔥 热门搜索</h2>
      <div class="flex flex-wrap gap-2">
        <span
          v-for="tag in hotTags"
          :key="tag.searchWord"
          @click="searchTag(tag.searchWord)"
          class="px-3 py-1.5 rounded-full bg-muted hover:bg-emphasis cursor-pointer transition text-sm"
        >
          {{ tag.searchWord }}
        </span>
      </div>
    </div>

    <div v-if="hasSearched">
      <div class="flex items-center gap-1 mb-4 bg-muted rounded-lg p-1 w-fit">
        <button v-for="tab in tabs" :key="tab.type" @click="switchTab(tab.type)"
          :class="['px-4 py-1.5 rounded-md text-sm font-medium transition', activeTab === tab.type ? 'bg-surface text-content shadow-sm' : 'text-content-2 hover:text-content']">
          {{ tab.label }}
          <span v-if="resultCache.has(tab.type) && resultCache.get(tab.type)!.count > 0" class="text-xs text-content-3 ml-1">{{ resultCache.get(tab.type)!.count }}</span>
        </button>
      </div>

      <div v-if="loading" class="flex items-center justify-center py-12">
        <div class="flex items-end gap-1 h-6">
          <span class="eq-bar w-[3px] bg-accent rounded-full" style="animation-delay: 0s"></span>
          <span class="eq-bar w-[3px] bg-accent rounded-full" style="animation-delay: 0.12s"></span>
          <span class="eq-bar w-[3px] bg-accent rounded-full" style="animation-delay: 0.24s"></span>
        </div>
      </div>

      <template v-else>
        <div v-if="activeTab === 1">
          <div v-if="currentResults.length" class="space-y-2">
            <SongListItem
              v-for="(song, index) in currentResults"
              :key="song.id"
              :song="song"
              :index="index"
              show-download
              show-menu
              cover-size="w-12 h-12"
              container-class="bg-subtle hover:bg-muted border border-line-2"
              @click="player.playFromList(currentResults, index)"
            />
          </div>
          <p v-else class="text-content-2 text-center py-8">{{ cacheError ? '搜索失败，点击其他标签页刷新重试' : '未找到相关歌曲' }}</p>
        </div>

        <div v-else-if="activeTab === 100">
          <div v-if="currentResults.length" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            <div v-for="artist in currentResults" :key="artist.id" @click="router.push({ name: 'artist', params: { id: artist.id } })"
              class="bg-subtle hover:bg-muted border border-line-2 rounded-xl p-4 cursor-pointer transition flex items-center gap-3">
              <img v-if="artist.picUrl" :src="artist.picUrl + '?param=100y100'" class="w-14 h-14 rounded-full object-cover flex-shrink-0" />
              <div v-else class="w-14 h-14 rounded-full bg-muted flex items-center justify-center flex-shrink-0">
                <IconUserRound class="w-5 h-5 text-content-3" />
              </div>
              <div class="min-w-0">
                <p class="text-sm font-medium truncate">{{ artist.name }}</p>
                <p v-if="artist.alias?.length" class="text-xs text-content-3 truncate">{{ artist.alias[0] }}</p>
                <p v-if="artist.musicSize" class="text-xs text-content-3">{{ artist.musicSize }} 首歌曲</p>
              </div>
            </div>
          </div>
          <p v-else class="text-content-2 text-center py-8">{{ cacheError ? '搜索失败，点击其他标签页刷新重试' : '未找到相关歌手' }}</p>
        </div>

        <div v-else-if="activeTab === 10">
          <div v-if="currentResults.length" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
            <div v-for="album in currentResults" :key="album.id" @click="router.push({ name: 'album', params: { id: album.id } })"
              class="bg-subtle hover:bg-muted border border-line-2 rounded-xl overflow-hidden cursor-pointer transition">
              <img v-if="album.picUrl" :src="album.picUrl + '?param=200y200'" class="w-full aspect-square object-cover" />
              <div v-else class="w-full aspect-square bg-muted flex items-center justify-center">
                <IconDisc class="w-8 h-8 text-content-3" />
              </div>
              <div class="p-3">
                <p class="text-sm font-medium truncate">{{ album.name }}</p>
                <p class="text-xs text-content-2 truncate mt-0.5">{{ album.artist?.name || '' }}</p>
                <p v-if="album.publishTime" class="text-xs text-content-3 mt-0.5">{{ formatDate(album.publishTime) }}</p>
              </div>
            </div>
          </div>
          <p v-else class="text-content-2 text-center py-8">{{ cacheError ? '搜索失败，点击其他标签页刷新重试' : '未找到相关专辑' }}</p>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DiscoverView' });

import { ref, computed, onMounted, onActivated, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { MusicApi } from '../api';
import { usePlayerStore } from '../stores/player';
import { useSettingsStore } from '../stores/settings';
import SongListItem from '../components/SongListItem.vue';
import { normalizeSong, type Song } from '../utils/song';
import { formatDate } from '../utils/format';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { useOnlineStatus } from '../composables/useOnlineStatus';
import IconUserRound from '~icons/lucide/user-round';
import IconDisc from '~icons/lucide/disc';

const router = useRouter();
const route = useRoute();
const player = usePlayerStore();
const settings = useSettingsStore();
const { isOnline } = useOnlineStatus();

/** 外部音源歌曲 ID 计数器（生成唯一负数占位 ID） */
let externalIdCounter = -1;
/** 将后端 ExternalSong 转为前端 Song 结构 */
function externalSongToSong(ext: any): Song {
  const artistStr: string = ext.artist || '';
  const ar = artistStr
    ? artistStr.split(/[\/,;&|]/).map((name: string) => ({ name: name.trim() })).filter((a: any) => a.name)
    : [];
  return {
    id: externalIdCounter--,
    name: ext.name || '未知歌曲',
    ar,
    al: { picUrl: ext.pic_url || '', name: ext.album || undefined },
    dt: ext.duration || 0,
    externalId: String(ext.id || ''),
    externalSourceId: ext.source || '',
    sourceLabel: ext.source_label || '',
  };
}

const loading = ref(false);
const hasSearched = ref(false);
const hotTags = ref<any[]>([]);
const activeTab = ref(1);
const cacheError = ref(false);

interface CacheEntry {
  data: Song[] | any[];
  count: number;
  dirty: boolean;
}

const resultCache = ref<Map<number, CacheEntry>>(new Map());
const lastSearchKeyword = ref('');

const currentResults = computed(() => {
  const entry = resultCache.value.get(activeTab.value);
  return entry ? entry.data : [];
});

const tabs = [
  { type: 1, label: '歌曲' },
  { type: 100, label: '歌手' },
  { type: 10, label: '专辑' },
];

async function loadHotTags() {
  const cached = pageCacheGet('discover_hotTags');
  if (cached) {
    hotTags.value = cached;
  } else {
    try {
      const json = await MusicApi.getHotSearch();
      const data = JSON.parse(json as string);
      hotTags.value = (data.data || []).slice(0, 12);
      pageCacheSet('discover_hotTags', hotTags.value);
    } catch { /* 忽略 */ }
  }
}

onMounted(async () => {
  await loadHotTags();
});

onActivated(async () => {
  if (pageCacheIsStale('discover_hotTags')) loadHotTags();
});

// 监听路由搜索参数变化（immediate 处理首次加载，_t 时间戳确保相同关键词也能重复搜索）
watch(() => [route.query.q, route.query._t], async ([q]) => {
  if (q && typeof q === 'string') {
    await handleSearch(q);
  }
}, { immediate: true });

watch(isOnline, (val, old) => {
  if (val && !old && hotTags.value.length === 0) {
    pageCacheInvalidate('discover_hotTags');
    loadHotTags();
  }
});

async function handleSearch(q: string) {
  const query = q.trim();
  if (!query) return;
  hasSearched.value = true;
  cacheError.value = false;
  lastSearchKeyword.value = query;
  resultCache.value.clear();

  await Promise.all([
    fetchTabResults(1),
    fetchTabResults(100),
    fetchTabResults(10),
  ]);
}

async function fetchTabResults(type: number) {
  const dedupeKey = (s: Song) => `${s.name.toLowerCase()}|${(s.ar?.[0]?.name || '').toLowerCase()}`;
  const entry = resultCache.value.get(type);
  if (entry && !entry.dirty) return;

  loading.value = true;
  cacheError.value = false;
  try {
    // 网易云搜索
    const cloudsearchPromise = MusicApi.cloudsearch({
      keyword: lastSearchKeyword.value, searchType: type, limit: 30
    });

    // 多源聚合搜索（仅歌曲 tab，且音源补充开启且有启用源时）
    const canMultiSearch = type === 1
      && settings.musicSourceEnabled
      && settings.enabledMusicSources.length > 0;
    const multiSearchPromise = canMultiSearch
      ? MusicApi.searchSongsMulti({
          keyword: lastSearchKeyword.value,
          sources: settings.enabledMusicSources,
          limit: 20,
        }).catch(() => '[]')
      : Promise.resolve('[]');

    const [jsonStr, externalJsonStr] = await Promise.all([cloudsearchPromise, multiSearchPromise]);

    const data = JSON.parse(jsonStr);
    const result = data.result || {};

    let items: any[] = [];
    if (type === 1) {
      const neteaseSongs: Song[] = (result.songs || []).map(normalizeSong);
      // 外部音源结果（如 QQ 音乐独占歌曲，通过 bodian/kugou/kuwo 等源搜到）
      const externalSongs: Song[] = (JSON.parse(externalJsonStr) || []).map(externalSongToSong);
      // 去重1：外部源中与网易云「歌名+歌手」都匹配的歌曲跳过，保留网易云版本
      // 只按歌名匹配会误杀——例如网易云有别的「定玄」，会把酷我的原版也过滤掉
      const neteaseKeys = new Set(neteaseSongs.map(dedupeKey));
      const externalFiltered = externalSongs.filter(s => {
        const key = dedupeKey(s);
        return !neteaseKeys.has(key);
      });
      // 去重2：外部源之间也可能搜到同一首（如 kuwo 和 bodian 都有），保留第一个
      const seenExternal = new Set<string>();
      const dedupedExternal = externalFiltered.filter(s => {
        const key = dedupeKey(s);
        if (seenExternal.has(key)) return false;
        seenExternal.add(key);
        return true;
      });
      items = [...neteaseSongs, ...dedupedExternal];
    } else if (type === 100) {
      items = result.artists || [];
    } else if (type === 10) {
      items = result.albums || [];
    }

    resultCache.value.set(type, { data: items, count: items.length, dirty: false });
  } catch (e) {
    console.error('搜索出错：', e);
    resultCache.value.set(type, { data: [], count: 0, dirty: true });
    cacheError.value = true;
  } finally {
    loading.value = false;
  }
}

async function switchTab(type: number) {
  if (type === activeTab.value) return;
  activeTab.value = type;

  const entry = resultCache.value.get(type);
  if (!entry || entry.dirty) {
    await fetchTabResults(type);
  }
}

function searchTag(tag: string) {
  // 通过路由跳转触发搜索（带时间戳确保相同关键词也能重复搜索）
  router.push({ name: 'discover', query: { q: tag, _t: String(Date.now()) } });
}
</script>
