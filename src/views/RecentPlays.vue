<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">最近播放</h1>
    </PageHeader>
    <div v-if="player.recentLocal.length === 0" class="text-content-3">还没有播放记录，去听首歌吧</div>
    <VirtualSongList
      v-else
      :songs="player.recentLocal"
      :current-song-id="player.currentSong?.id"
      show-index
      show-like
      show-download
      show-menu
      show-duration
      show-playing-overlay
      @song-click="(_s, i) => player.playFromList(player.recentLocal, i)"
    />
  </div>
</template>

<script setup lang="ts">
import { usePlayerStore } from '../stores/player';
import VirtualSongList from '../components/VirtualSongList.vue';
import PageHeader from '../components/PageHeader.vue';

const player = usePlayerStore();
</script>
