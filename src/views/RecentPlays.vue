<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">最近播放</h1>
    </PageHeader>
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
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore } from '../stores/player';
import SongListItem from '../components/SongListItem.vue';
import PageHeader from '../components/PageHeader.vue';

const player = usePlayerStore();
</script>
