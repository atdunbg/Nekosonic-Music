<template>
  <div class="p-8 text-content">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <h1 class="text-2xl font-bold mb-6">最近播放</h1>
    <div v-if="player.recentLocal.length === 0" class="text-content-3">还没有播放记录，去听首歌吧</div>
    <div v-else class="space-y-2">
      <SongListItem
        v-for="(song, index) in player.recentLocal"
        :key="song.id"
        :song="song"
        :index="index"
        :is-current="player.currentSong?.id === song.id"
        show-index
        show-like
        show-download
        show-menu
        show-duration
        show-playing-overlay
        :container-class="player.currentSong?.id === song.id ? 'bg-accent-dim hover:bg-accent-dim' : 'hover:bg-subtle'"
        @click="player.playFromList(player.recentLocal, index)"
      >
        <template #index="{ index: idx, isCurrent }">
          <div class="w-6 text-right flex-shrink-0 flex items-center justify-end h-5">
            <div v-if="isCurrent" class="flex items-center justify-end">
              <div class="flex items-center gap-[3px] h-4">
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 50%; animation-delay: 0ms"></span>
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 100%; animation-delay: 150ms"></span>
                <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 35%; animation-delay: 300ms"></span>
              </div>
            </div>
            <template v-else>
              <span class="text-xs text-content-3 group-hover:hidden">{{ idx + 1 }}</span>
              <svg class="hidden group-hover:block text-content" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2.5v11l9-5.5z"/></svg>
            </template>
          </div>
        </template>
      </SongListItem>
    </div>
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore } from '../stores/player';
import SongListItem from '../components/SongListItem.vue';

const player = usePlayerStore();
</script>
