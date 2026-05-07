<template>
  <div class="p-8 text-white">
    <h1 class="text-2xl font-bold mb-4">发现音乐</h1>

    <!-- 搜索框 -->
    <input
      v-model="keyword"
      @keyup.enter="handleSearch"
      placeholder="搜索歌曲、歌手、专辑..."
      class="mb-4 w-full rounded-xl bg-white/10 p-3 text-white placeholder-gray-400 outline-none backdrop-blur"
    />

    <!-- 热门搜索标签（仅在没有搜索且未显示结果时出现） -->
    <div v-if="!hasSearched && !loading && hotTags.length" class="mb-6">
      <h2 class="text-sm font-semibold mb-3">🔥 热门搜索</h2>
      <div class="flex flex-wrap gap-2">
        <span
          v-for="tag in hotTags"
          :key="tag.searchWord"
          @click="searchTag(tag.searchWord)"
          class="px-3 py-1 rounded-full bg-white/10 hover:bg-white/20 cursor-pointer transition text-sm"
        >
          {{ tag.searchWord }}
        </span>
      </div>
    </div>

    <!-- 输出设备选择 -->
    <!-- <div class="mb-4">
      <label class="mr-2 text-sm text-gray-400">输出设备：</label>
      <select v-model="selectedDevice" @change="changeDevice" class="bg-white/10 text-white rounded p-1 text-sm">
        <option :value="null">跟随系统默认</option>
        <option v-for="dev in devices" :key="dev" :value="dev">{{ dev }}</option>
      </select>
    </div> -->

    <!-- 搜索结果 -->
    <div v-if="loading" class="text-gray-400">搜索中...</div>
    <div v-else class="space-y-3">
      <div
        v-for="song in results"
        :key="song.id"
        @click="playSong(song)"
        class="flex items-center gap-4 p-3 rounded-xl backdrop-blur-md bg-white/5 hover:bg-white/10 border border-white/5 cursor-pointer transition"
      >
        <img :src="song.al?.picUrl" class="w-12 h-12 rounded-lg object-cover" />
        <div>
          <p class="font-medium">{{ song.name }}</p>
          <p class="text-sm text-gray-400">
            {{ song.ar?.map((a: any) => a.name).join(' / ') }}
          </p>
        </div>
      </div>
      <p v-if="!loading && hasSearched && results.length === 0" class="text-gray-400">无结果</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'DiscoverView' });

import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { usePlayerStore } from '../stores/player';

const router = useRouter();
const route = useRoute();
const player = usePlayerStore();

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

async function playSong(song: any) {
  player.play(song);
}

</script>