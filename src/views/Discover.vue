<template>
  <div class="p-8 text-content" @click="showSuggestions = false">
    <div class="relative mb-6" @click.stop>
      <div class="flex items-center gap-3">
        <div class="relative flex-1">
          <IconSearch class="absolute left-3.5 top-1/2 -translate-y-1/2 text-content-3 w-[18px] h-[18px]" />
          <input
            ref="searchInput"
            v-model="keyword"
            @input="onInputChange"
            @keydown.enter="handleSearch"
            @focus="onInputFocus"
            placeholder="搜索歌曲、歌手、专辑..."
            class="w-full rounded-xl bg-muted pl-10 pr-10 py-3 text-content placeholder-content-3 outline-none focus:bg-subtle focus:ring-1 focus:ring-accent/30 transition"
          />
          <button v-if="keyword" @click="clearSearch" class="absolute right-3 top-1/2 -translate-y-1/2 text-content-3 hover:text-content transition">
            <IconX class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div v-if="showSuggestions && !hasSearched"
        class="absolute z-30 left-0 right-0 top-full mt-2 bg-surface border border-line-2 rounded-xl shadow-xl overflow-hidden max-h-[60vh] overflow-y-auto">
        <div v-if="suggestions.length" class="p-2">
          <p class="text-xs text-content-3 px-3 py-1.5">搜索建议</p>
          <button v-for="s in suggestions" :key="s" @click="searchTag(s)"
            class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-muted transition flex items-center gap-2">
            <IconSearch style="font-size: 14px" class="text-content-3 flex-shrink-0" />
            <span>{{ s }}</span>
          </button>
        </div>
        <div v-if="searchHistory.length && !suggestions.length" class="p-2">
          <div class="flex items-center justify-between px-3 py-1.5">
            <p class="text-xs text-content-3">搜索历史</p>
            <button @click.stop="clearHistory" class="text-xs text-content-3 hover:text-danger transition">清空</button>
          </div>
          <button v-for="h in searchHistory" :key="h" @click="searchTag(h)"
            class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-muted transition flex items-center gap-2">
            <IconHistory style="font-size: 14px" class="text-content-3 flex-shrink-0" />
            <span>{{ h }}</span>
          </button>
        </div>
        <div v-if="hotTags.length && !suggestions.length && !searchHistory.length" class="p-2">
          <p class="text-xs text-content-3 px-3 py-1.5">热门搜索</p>
          <button v-for="tag in hotTags" :key="tag.searchWord" @click="searchTag(tag.searchWord)"
            class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-muted transition flex items-center gap-2">
            <IconClock style="font-size: 14px" class="text-content-3 flex-shrink-0" />
            <span>{{ tag.searchWord }}</span>
          </button>
        </div>
      </div>
    </div>

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

import { ref, computed, onMounted, onActivated, watch, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import SongListItem from '../components/SongListItem.vue';
import { normalizeSong, type Song } from '../utils/song';
import { formatDate } from '../utils/format';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { useOnlineStatus } from '../composables/useOnlineStatus';
import IconSearch from '~icons/lucide/search';
import IconX from '~icons/lucide/x';
import IconHistory from '~icons/lucide/history';
import IconClock from '~icons/lucide/clock';
import IconUserRound from '~icons/lucide/user-round';
import IconDisc from '~icons/lucide/disc';

const router = useRouter();
const route = useRoute();
const player = usePlayerStore();
const { isOnline } = useOnlineStatus();

const searchInput = ref<HTMLInputElement | null>(null);
const keyword = ref('');
const loading = ref(false);
const hasSearched = ref(false);
const hotTags = ref<any[]>([]);
const suggestions = ref<string[]>([]);
const showSuggestions = ref(false);
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

const HISTORY_KEY = 'search_history';
const MAX_HISTORY = 15;

function loadSearchHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* 忽略 */ }
  return [];
}

function saveSearchHistory(q: string) {
  let history = loadSearchHistory();
  history = history.filter(h => h !== q);
  history.unshift(q);
  if (history.length > MAX_HISTORY) history = history.slice(0, MAX_HISTORY);
  localStorage.setItem(HISTORY_KEY, JSON.stringify(history));
}

const searchHistory = ref<string[]>(loadSearchHistory());

function clearHistory() {
  searchHistory.value = [];
  localStorage.removeItem(HISTORY_KEY);
}

let suggestTimer: ReturnType<typeof setTimeout> | null = null;

function onInputChange() {
  if (suggestTimer) clearTimeout(suggestTimer);
  if (!keyword.value.trim()) {
    suggestions.value = [];
    showSuggestions.value = true;
    return;
  }
  suggestTimer = setTimeout(async () => {
    try {
      const jsonStr: string = await invoke('search_suggest', { query: { keyword: keyword.value.trim() } });
      const data = JSON.parse(jsonStr);
      const all = data.result?.allMatch || [];
      suggestions.value = all.map((m: any) => m.keyword).slice(0, 8);
      showSuggestions.value = true;
    } catch {
      suggestions.value = [];
    }
  }, 300);
}

function onInputFocus() {
  if (!hasSearched.value) {
    showSuggestions.value = true;
  }
}

async function loadHotTags() {
  const cached = pageCacheGet('discover_hotTags');
  if (cached) {
    hotTags.value = cached;
  } else {
    try {
      const json = await invoke('get_hot_search');
      const data = JSON.parse(json as string);
      hotTags.value = (data.data || []).slice(0, 12);
      pageCacheSet('discover_hotTags', hotTags.value);
    } catch { /* 忽略 */ }
  }
}

onMounted(async () => {
  await loadHotTags();
  const q = route.query.q as string;
  if (q) {
    keyword.value = q;
    await handleSearch();
    router.replace({ query: {} });
  }
});

onActivated(async () => {
  if (pageCacheIsStale('discover_hotTags')) loadHotTags();
  const q = route.query.q as string;
  if (q && q !== lastSearchKeyword.value) {
    keyword.value = q;
    await handleSearch();
    router.replace({ query: {} });
  }
});

watch(isOnline, (val, old) => {
  if (val && !old && hotTags.value.length === 0) {
    pageCacheInvalidate('discover_hotTags');
    loadHotTags();
  }
});

async function handleSearch() {
  const q = keyword.value.trim();
  if (!q) return;
  showSuggestions.value = false;
  hasSearched.value = true;
  cacheError.value = false;
  saveSearchHistory(q);
  searchHistory.value = loadSearchHistory();

  if (q === lastSearchKeyword.value && resultCache.value.size > 0) return;

  lastSearchKeyword.value = q;
  resultCache.value.clear();

  await Promise.all([
    fetchTabResults(1),
    fetchTabResults(100),
    fetchTabResults(10),
  ]);
}

async function fetchTabResults(type: number) {
  const entry = resultCache.value.get(type);
  if (entry && !entry.dirty) return;

  loading.value = true;
  cacheError.value = false;
  try {
    const jsonStr: string = await invoke('cloudsearch', {
      query: { keyword: lastSearchKeyword.value, searchType: type, limit: 30 }
    });
    const data = JSON.parse(jsonStr);
    const result = data.result || {};

    let items: any[] = [];
    if (type === 1) {
      items = (result.songs || []).map(normalizeSong);
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
  keyword.value = tag;
  handleSearch();
}

function clearSearch() {
  keyword.value = '';
  hasSearched.value = false;
  resultCache.value.clear();
  lastSearchKeyword.value = '';
  cacheError.value = false;
  suggestions.value = [];
  showSuggestions.value = true;
  nextTick(() => searchInput.value?.focus());
}
</script>
