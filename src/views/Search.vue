<template>
  <div class="text-white">
    <h1 class="text-2xl font-bold mb-4">搜索</h1>

    <!-- 输出设备选择-->
    <!-- <div class="mb-4">
      <label class="mr-2">输出设备：</label>
      <select v-model="selectedDevice" @change="changeDevice" class="bg-white/10 text-white rounded p-1">
        <option :value="null">跟随系统默认</option>
        <option v-for="dev in devices" :key="dev" :value="dev">{{ dev }}</option>
      </select>
    </div> -->

    <input
      v-model="keyword"
      @keyup.enter="handleSearch"
      placeholder="搜索歌曲..."
      class="mb-6 w-full rounded-xl bg-white/10 p-3 text-white placeholder-gray-400 outline-none backdrop-blur"
    />

    <div v-if="loading" class="text-gray-400">搜索中...</div>
    <div v-else class="space-y-3">
      <div
        v-for="song in results"
        :key="song.id"
        @click="playSong(song)"
        class="flex items-center gap-4 p-3 rounded-xl backdrop-blur-md bg-white/5 hover:bg-white/10 border border-white/5 cursor-pointer transition-all duration-200 hover:scale-[1.01] active:scale-95"
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
// 监听从首页或其他地方传来的 query 参数，自动搜索
watch(
  () => route.query.q,
  (newQ) => {
    if (newQ) {
      keyword.value = newQ as string;
      handleSearch();
      // 清除 query，防止刷新后重复搜索
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
// const selectedDevice = ref<string | null>(null);

onMounted(async () => {
  devices.value = await invoke('get_output_devices');
});

// async function changeDevice() {
//   await invoke('set_output_device', { device: selectedDevice.value });
// }
</script>