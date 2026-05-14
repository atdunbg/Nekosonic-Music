<template>
  <div class="text-content">
    <h1 class="text-2xl font-bold mb-4">搜索</h1>

    <input
      v-model="keyword"
      @keyup.enter="handleSearch"
      placeholder="搜索歌曲..."
      class="mb-6 w-full rounded-xl bg-muted p-3 text-content placeholder-content-2 outline-none backdrop-blur"
    />

    <div v-if="loading" class="text-content-2">搜索中...</div>
    <div v-else class="space-y-3">
      <div
        v-for="(song, index) in results"
        :key="song.id"
        @click="playSong(song, index)"
        class="flex items-center gap-4 p-3 rounded-xl backdrop-blur-md bg-subtle hover:bg-muted border border-line-2 cursor-pointer transition-all duration-200 hover:scale-[1.01] active:scale-95"
      >
        <img :src="song.al?.picUrl" class="w-12 h-12 rounded-lg object-cover" />
        <div class="flex-1 min-w-0">
          <p class="font-medium truncate">{{ song.name }}</p>
          <p class="text-sm text-content-2 truncate">
            {{ song.ar?.map((a: any) => a.name).join(' / ') }}
          </p>
        </div>
        <button @click.stop="download.downloadSong(song)" class="text-content-3 hover:text-accent-text transition flex-shrink-0" :title="download.isDownloaded(song.id) ? '已下载' : '下载'">
          <svg v-if="download.isDownloading(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
          <svg v-else-if="download.isDownloaded(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent-text"><polyline points="20 6 9 17 4 12"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </button>
      </div>
      <p v-if="!loading && hasSearched && results.length === 0" class="text-content-2">无结果</p>
    </div>

  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'SearchView' });

import { useRoute } from 'vue-router';
import { watch } from 'vue';
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import { useRouter } from 'vue-router';
const router = useRouter();

const keyword = ref('');
const results = ref<any[]>([]);
const loading = ref(false);
const hasSearched = ref(false);
const player = usePlayerStore();
const download = useDownload();

const route = useRoute();
watch(
  () => route.query.q,
  (newQ) => {
    if (newQ) {
      keyword.value = newQ as string;
      handleSearch();
      router.replace({ query: {} });
    }
  },
  { immediate: true }
);

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

async function playSong(_song: any, index: number) {
  try {
    const normalized = results.value.map((s: any) => ({
      id: s.id,
      name: s.name,
      ar: s.ar || s.artists || [],
      al: s.al || s.album || { picUrl: '' },
      dt: s.dt || 0,
    }));
    await player.playFromList(normalized, index);
  } catch (e) {
    alert('暂无播放源或需登录');
  }
}

const devices = ref<string[]>([]);

onMounted(async () => {
  devices.value = await invoke('get_output_devices');
});
</script>
