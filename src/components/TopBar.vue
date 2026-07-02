<template>
  <div
    class="h-12 flex items-center px-4 gap-3 flex-shrink-0 select-none relative z-20 border-b border-line-2/40"
    :style="barBgStyle"
  >
    <!-- 抽屉模式：菜单触发按钮 -->
    <button
      v-if="ui.sidebarMode === 'drawer'"
      @click="ui.toggleDrawer()"
      class="w-8 h-8 flex items-center justify-center rounded-lg text-content-2 hover:text-content hover:bg-muted transition flex-shrink-0"
      title="打开菜单"
    >
      <IconMenu class="w-5 h-5" />
    </button>

    <!-- 前进/后退 -->
    <div class="flex items-center gap-1 flex-shrink-0">
      <button
        @click="goBack"
        :disabled="!canGoBack"
        :class="[
          'w-7 h-7 flex items-center justify-center rounded-full transition',
          canGoBack ? 'text-content-2 hover:text-content hover:bg-muted' : 'text-content-3/40 cursor-not-allowed'
        ]"
        title="后退"
      >
        <IconArrowLeft class="w-[18px] h-[18px]" />
      </button>
      <button
        @click="goForward"
        :disabled="!canGoForward"
        :class="[
          'w-7 h-7 flex items-center justify-center rounded-full transition',
          canGoForward ? 'text-content-2 hover:text-content hover:bg-muted' : 'text-content-3/40 cursor-not-allowed'
        ]"
        title="前进"
      >
        <IconArrowRight class="w-[18px] h-[18px]" />
      </button>
    </div>

    <!-- 可拖拽空白区（窗口拖动） -->
    <div class="flex-1 h-full" data-tauri-drag-region></div>

    <!-- 搜索框 -->
    <div class="relative w-80 flex-shrink-0" @click.stop>
      <IconSearch class="absolute left-3 top-1/2 -translate-y-1/2 text-content-3 w-4 h-4 pointer-events-none" />
      <input
        ref="searchInputRef"
        v-model="keyword"
        @input="onInputChange"
        @keydown.enter="handleSearch"
        @focus="onInputFocus"
        placeholder="搜索歌曲、歌手、专辑..."
        class="w-full h-8 rounded-full bg-muted/70 pl-9 pr-8 text-sm text-content placeholder-content-3 outline-none focus:bg-muted focus:ring-1 focus:ring-accent/40 transition"
      />
      <button
        v-if="keyword"
        @click="clearKeyword"
        class="absolute right-2.5 top-1/2 -translate-y-1/2 text-content-3 hover:text-content transition"
      >
        <IconX class="w-3.5 h-3.5" />
      </button>

      <!-- 下拉面板 -->
      <Transition name="fade">
        <div
          v-if="showPanel"
          class="absolute z-30 left-0 right-0 top-full mt-1.5 bg-surface border border-line-2 rounded-xl shadow-xl overflow-hidden max-h-[60vh] overflow-y-auto"
        >
          <!-- 搜索建议 -->
          <div v-if="suggestions.length" class="p-1.5">
            <p class="text-xs text-content-3 px-3 py-1.5">搜索建议</p>
            <button
              v-for="s in suggestions"
              :key="s"
              @click="searchTag(s)"
              class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-muted transition flex items-center gap-2"
            >
              <IconSearch class="w-3.5 h-3.5 text-content-3 flex-shrink-0" />
              <span>{{ s }}</span>
            </button>
          </div>

          <!-- 搜索历史 -->
          <div v-if="!suggestions.length && searchHistory.length" class="p-1.5">
            <div class="flex items-center justify-between px-3 py-1.5">
              <p class="text-xs text-content-3">搜索历史</p>
              <button @click.stop="clearHistory" class="text-xs text-content-3 hover:text-danger transition">清空</button>
            </div>
            <button
              v-for="h in searchHistory"
              :key="h"
              @click="searchTag(h)"
              class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-muted transition flex items-center gap-2"
            >
              <IconHistory class="w-3.5 h-3.5 text-content-3 flex-shrink-0" />
              <span>{{ h }}</span>
            </button>
          </div>

          <!-- 热门搜索 -->
          <div v-if="!suggestions.length && !searchHistory.length && hotTags.length" class="p-1.5">
            <p class="text-xs text-content-3 px-3 py-1.5">热门搜索</p>
            <button
              v-for="tag in hotTags"
              :key="tag.searchWord"
              @click="searchTag(tag.searchWord)"
              class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-muted transition flex items-center gap-2"
            >
              <IconFlame class="w-3.5 h-3.5 text-content-3 flex-shrink-0" />
              <span>{{ tag.searchWord }}</span>
            </button>
          </div>
        </div>
      </Transition>
    </div>

    <!-- 可拖拽空白区（窗口拖动） -->
    <div class="flex-1 h-full" data-tauri-drag-region></div>

    <!-- 右侧：窗口控制 -->
    <div class="flex items-center gap-1.5 flex-shrink-0">
      <button @click="minimizeWindow" class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition" title="最小化"></button>
      <button @click="toggleMaximize" class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition" title="最大化/还原"></button>
      <button @click="$emit('close')" class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition" title="关闭"></button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { MusicApi } from '../api';
import { useUiStore } from '../stores/ui';
import { pageCacheGet, pageCacheSet, pageCacheIsStale, pageCacheInvalidate } from '../composables/usePageCache';
import { minimizeWindow, toggleMaximize } from '../composables/useWindowControls';
import IconArrowLeft from '~icons/lucide/arrow-left';
import IconArrowRight from '~icons/lucide/arrow-right';
import IconSearch from '~icons/lucide/search';
import IconX from '~icons/lucide/x';
import IconHistory from '~icons/lucide/history';
import IconFlame from '~icons/lucide/flame';
import IconMenu from '~icons/lucide/menu';

defineEmits<{ close: [] }>();

const router = useRouter();
const route = useRoute();
const ui = useUiStore();

// --- 导航历史 ---
const canGoBack = ref(false);
const canGoForward = ref(false);

function updateNavState() {
  // window.history.length 在 SPA 中不可靠，用自定义栈判断
  canGoBack.value = navStack.value.length > 1;
  canGoForward.value = navStack.value.length > 0 && navStackIdx.value < navStack.value.length - 1;
}

const navStack = ref<string[]>([]);
const navStackIdx = ref(-1);

function goBack() {
  if (!canGoBack.value) return;
  router.go(-1);
}
function goForward() {
  if (!canGoForward.value) return;
  router.go(1);
}

// --- 搜索 ---
const searchInputRef = ref<HTMLInputElement | null>(null);
const keyword = ref('');
const suggestions = ref<string[]>([]);
const showPanel = ref(false);
const hotTags = ref<any[]>([]);

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
    showPanel.value = true;
    return;
  }
  suggestTimer = setTimeout(async () => {
    try {
      const jsonStr: string = await MusicApi.searchSuggest(keyword.value.trim());
      const data = JSON.parse(jsonStr);
      const result = data.result || {};
      // 优先用 allMatch（type=mobile 格式）
      let list: string[] = (result.allMatch || []).map((m: any) => m.keyword);
      // 兜底：从 songs/artists/albums 提取名称
      if (list.length === 0) {
        list = [
          ...(result.songs || []).map((s: any) => s.name),
          ...(result.artists || []).map((a: any) => a.name),
          ...(result.albums || []).map((a: any) => a.name),
        ].filter(Boolean);
      }
      suggestions.value = list.slice(0, 8);
      showPanel.value = true;
    } catch {
      suggestions.value = [];
    }
  }, 300);
}

function onInputFocus() {
  showPanel.value = true;
}

function clearKeyword() {
  keyword.value = '';
  suggestions.value = [];
  showPanel.value = true;
  searchInputRef.value?.focus();
}

function searchTag(q: string) {
  keyword.value = q;
  handleSearch();
}

async function handleSearch() {
  const q = keyword.value.trim();
  if (!q) return;
  showPanel.value = false;
  saveSearchHistory(q);
  searchHistory.value = loadSearchHistory();
  // 跳转到 Discover 页面并带 query
  router.push({ name: 'discover', query: { q } });
}

async function loadHotTags() {
  const cached = pageCacheGet('topbar_hotTags');
  if (cached) {
    hotTags.value = cached;
  } else {
    try {
      const json = await MusicApi.getHotSearch();
      const data = JSON.parse(json as string);
      hotTags.value = (data.data || []).slice(0, 8);
      pageCacheSet('topbar_hotTags', hotTags.value);
    } catch { /* 忽略 */ }
  }
}

// --- 窗口控制（来自公共 composable） ---

// --- 背景样式 ---
// 透明背景：无壁纸时继承主容器，有壁纸时透出壁纸
const barBgStyle = computed(() => {
  return {};
});

// --- 路由监听：维护导航栈 + 点击外部关闭面板 ---
function onRouteChange() {
  const path = route.fullPath;
  if (navStackIdx.value >= 0 && navStack.value[navStackIdx.value] === path) return;

  // 如果是前进/后退，栈已存在该路径
  const existingIdx = navStack.value.indexOf(path);
  if (existingIdx !== -1) {
    navStackIdx.value = existingIdx;
  } else {
    // 新导航：截断前进历史
    navStack.value = navStack.value.slice(0, navStackIdx.value + 1);
    navStack.value.push(path);
    navStackIdx.value = navStack.value.length - 1;
  }
  updateNavState();
}

function onDocClick() {
  showPanel.value = false;
}

onMounted(() => {
  loadHotTags();
  onRouteChange();
  router.afterEach(onRouteChange);
  document.addEventListener('click', onDocClick);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick);
});

// 热门标签缓存过期刷新
onMounted(() => {
  if (pageCacheIsStale('topbar_hotTags')) {
    pageCacheInvalidate('topbar_hotTags');
    loadHotTags();
  }
});
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
