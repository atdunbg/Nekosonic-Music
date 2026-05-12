<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <h1 class="text-2xl font-bold mb-6">最近播放</h1>
    <div v-if="player.recentLocal.length === 0" class="text-content-3">还没有播放记录，去听首歌吧</div>
    <div v-else class="space-y-2">
      <div
        v-for="(song, index) in player.recentLocal"
        :key="song.id"
        @click="player.play(song)"
        class="flex items-center gap-4 p-3 rounded-xl hover:bg-subtle transition cursor-pointer"
      >
        <span class="text-xs text-content-3 w-6 text-right">{{ index + 1 }}</span>
        <img :src="song.al?.picUrl" class="w-10 h-10 rounded object-cover" />
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">{{ song.name }}</p>
          <p class="text-xs text-content-2 truncate">
            {{ song.ar?.map((a: any) => a.name).join(' / ') }}
          </p>
        </div>
        <button @click.stop="player.toggleLike(song.id)" class="text-content-3 hover:text-danger transition flex-shrink-0">
          <svg v-if="player.isLiked(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="text-danger"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
        </button>
        <span class="text-xs text-content-3">{{ formatDuration(song.dt ?? 0) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore } from '../stores/player';
import { formatDuration } from '../utils/format';

const player = usePlayerStore();
</script>
