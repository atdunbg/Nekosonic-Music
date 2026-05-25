<template>
  <div class="p-8 text-content">
    <h1 class="text-2xl font-bold mb-4">发现音乐</h1>

    <input
      v-model="keyword"
      @keyup.enter="handleSearch"
      placeholder="搜索歌曲、歌手、专辑..."
      class="mb-4 w-full rounded-xl bg-muted p-3 text-content placeholder-content-2 outline-none backdrop-blur"
    />

    <div v-if="!hasSearched && !loading && hotTags.length" class="mb-6">
      <h2 class="text-sm font-semibold mb-3">🔥 热门搜索</h2>
      <div class="flex flex-wrap gap-2">
        <span
          v-for="tag in hotTags"
          :key="tag.searchWord"
          @click="searchTag(tag.searchWord)"
          class="px-3 py-1 rounded-full bg-muted hover:bg-emphasis cursor-pointer transition text-sm"
        >
          {{ tag.searchWord }}
        </span>
      </div>
    </div>

    <div v-if="loading" class="text-content-2">搜索中...</div>
    <div v-else class="space-y-3">
      <SongListItem
        v-for="(song, index) in results"
        :key="song.id"
        :song="song"
        :index="index"
        show-download
        show-menu
        cover-size="w-12 h-12"
        container-class="backdrop-blur-md bg-subtle hover:bg-muted border border-line-2"
        @click="player.playFromList(results, index)"
      />
      <p v-if="!loading && hasSearched && results.length === 0" class="text-content-2">无结果</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DiscoverView' });

import { ref, onMounted, onActivated, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import SongListItem from '../components/SongListItem.vue';
import { normalizeSong, type Song } from '../utils/song';
import { pageCacheGet, pageCacheSet, pageCacheInvalidate, pageCacheIsStale } from '../composables/usePageCache';
import { useOnlineStatus } from '../composables/useOnlineStatus';

const router = useRouter();
const route = useRoute();
const player = usePlayerStore();
const { isOnline } = useOnlineStatus();

const keyword = ref('');
const results = ref<Song[]>([]);
const loading = ref(false);
const hasSearched = ref(false);
const hotTags = ref<any[]>([]);

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
    } catch {}
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

onActivated(() => {
  if (pageCacheIsStale('discover_hotTags')) loadHotTags();
});

watch(isOnline, (val, old) => {
  if (val && !old && hotTags.value.length === 0) {
    pageCacheInvalidate('discover_hotTags');
    loadHotTags();
  }
});

async function handleSearch() {
  if (!keyword.value.trim()) return;
  loading.value = true;
  hasSearched.value = true;
  try {
    const jsonStr: string = await invoke('search_songs', { query: { keyword: keyword.value } });
    const data = JSON.parse(jsonStr);
    results.value = (data.result?.songs || []).map(normalizeSong);
  } catch (e) {
    console.error('搜索出错：', e);
  } finally {
    loading.value = false;
  }
}

function searchTag(tag: string) {
  keyword.value = tag;
  handleSearch();
}
</script>
