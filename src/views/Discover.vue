<template>
  <div class="p-8 text-content">
    <h1 class="text-2xl font-bold mb-4">发现音乐</h1>

    <!-- 搜索框 -->
    <input
      v-model="keyword"
      @keyup.enter="handleSearch"
      placeholder="搜索歌曲、歌手、专辑..."
      class="mb-4 w-full rounded-xl bg-muted p-3 text-content placeholder-content-2 outline-none backdrop-blur"
    />

    <!-- 热门搜索标签（仅在没有搜索且未显示结果时出现） -->
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

    <!-- 输出设备选择 -->
    <!-- <div class="mb-4">
      <label class="mr-2 text-sm text-content-2">输出设备：</label>
      <select v-model="selectedDevice" @change="changeDevice" class="bg-muted text-white rounded p-1 text-sm">
        <option :value="null">跟随系统默认</option>
        <option v-for="dev in devices" :key="dev" :value="dev">{{ dev }}</option>
      </select>
    </div> -->

    <!-- 搜索结果 -->
    <div v-if="loading" class="text-content-2">搜索中...</div>
    <div v-else class="space-y-3">
      <div
        v-for="(song, index) in results"
        :key="song.id"
        @click="playSong(song, index)"
        class="flex items-center gap-4 p-3 rounded-xl backdrop-blur-md bg-subtle hover:bg-muted border border-line-2 cursor-pointer transition"
      >
        <img :src="song.al?.picUrl" class="w-12 h-12 rounded-lg object-cover" />
        <div class="flex-1 min-w-0">
          <p class="font-medium truncate">{{ song.name }}</p>
          <p class="text-sm text-content-2 truncate">
            <template v-for="(a, i) in song.ar || []" :key="a.id || i">
              <span v-if="i > 0" class="text-content-3">/</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click.stop="a.id && router.push({ name: 'artist', params: { id: a.id } })">{{ a.name }}</span>
            </template>
            <template v-if="song.al?.name">
              <span class="text-content-3 mx-1">·</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click.stop="song.al.id && router.push({ name: 'album', params: { id: song.al.id } })">{{ song.al.name }}</span>
            </template>
          </p>
        </div>
        <button @click.stop="download.downloadSong(song)" class="text-content-3 hover:text-accent-text transition flex-shrink-0" :title="download.isDownloaded(song.id) ? '已下载' : '下载'">
          <svg v-if="download.isDownloading(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
          <svg v-else-if="download.isDownloaded(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent-text"><polyline points="20 6 9 17 4 12"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </button>
        <SongItemMenu :song-id="song.id" />
      </div>
      <p v-if="!loading && hasSearched && results.length === 0" class="text-content-2">无结果</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DiscoverView' });

import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import SongItemMenu from '../components/SongItemMenu.vue';

const router = useRouter();
const route = useRoute();
const player = usePlayerStore();
const download = useDownload();

const keyword = ref('');
const results = ref<any[]>([]);
const loading = ref(false);
const hasSearched = ref(false);
const hotTags = ref<any[]>([]);

const devices = ref<string[]>([]);

onMounted(async () => {
  // 获取输出设备列表
  try { devices.value = await invoke('get_output_devices'); } catch {}

  // 获取热门搜索
  try {
    const json = await invoke('get_hot_search');
    const data = JSON.parse(json as string);
    hotTags.value = (data.data || []).slice(0, 12);
  } catch {}

  // 检查路由是否有查询关键词，自动搜索
  const q = route.query.q as string;
  if (q) {
    keyword.value = q;
    await handleSearch();
    router.replace({ query: {} });
  }
});

async function handleSearch() {
  if (!keyword.value.trim()) return;
  loading.value = true;
  hasSearched.value = true;
  try {
    const jsonStr: string = await invoke('search_songs', { query: { keyword: keyword.value } });
    const data = JSON.parse(jsonStr);
    results.value = data.result?.songs || [];
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

async function playSong(_song: any, index: number) {
  const normalized = results.value.map((s: any) => ({
    id: s.id,
    name: s.name,
    ar: s.ar || s.artists || [],
    al: s.al || s.album || { picUrl: '' },
    dt: s.dt || 0,
  }));
  player.playFromList(normalized, index);
}

</script>
