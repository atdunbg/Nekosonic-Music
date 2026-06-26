<template>
  <div class="flex flex-col">
    <SongListItem
      v-for="(song, index) in songs"
      :key="song.id"
      :song="song"
      :index="index"
      :is-current="song.id === currentSongId"
      :show-index="showIndex"
      :show-like="showLike"
      :show-download="showDownload"
      :show-menu="showMenu"
      :show-duration="showDuration"
      :show-playing-overlay="showPlayingOverlay"
      :cover-size="coverSize"
      :cover-size-param="coverSizeParam"
      @click="$emit('song-click', song, index)"
    />
  </div>
</template>

<script setup lang="ts">
import SongListItem from './SongListItem.vue';
import type { Song } from '../types/song';

withDefaults(defineProps<{
  songs: Song[];
  currentSongId?: number;
  showIndex?: boolean;
  showLike?: boolean;
  showDownload?: boolean;
  showMenu?: boolean;
  showDuration?: boolean;
  showPlayingOverlay?: boolean;
  coverSize?: string;
  coverSizeParam?: string;
}>(), {
  showIndex: false,
  showLike: false,
  showDownload: false,
  showMenu: false,
  showDuration: false,
  showPlayingOverlay: false,
  coverSize: 'w-10 h-10',
  coverSizeParam: '',
});

defineEmits<{
  (e: 'song-click', song: Song, index: number): void;
}>();
</script>
