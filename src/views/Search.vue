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
        v-for="song in results"
        :key="song.id"
        @click="playSong(song)"
        class="flex items-center gap-4 p-3 rounded-xl backdrop-blur-md bg-subtle hover:bg-muted border border-line-2 cursor-pointer transition-all duration-200 hover:scale-[1.01] active:scale-95"
      >
        <img :src="song.al?.picUrl" class="w-12 h-12 rounded-lg object-cover" />
        <div>
          <p class="font-medium">{{ song.name }}</p>
          <p class="text-sm text-content-2">
            {{ song.ar?.map((a: any) => a.name).join(' / ') }}
          </p>
        </div>
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
import { useRouter } from 'vue-router';
const router = useRouter();

const keyword = ref('');
const results = ref<any[]>([]);
const loading = ref(false);
const hasSearched = ref(false);
const player = usePlayerStore();

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

async function playSong(song: any) {
  try {
    await player.play(song);
  } catch (e) {
    alert('暂无播放源或需登录');
  }
}

const devices = ref<string[]>([]);

onMounted(async () => {
  devices.value = await invoke('get_output_devices');
});
</script>
