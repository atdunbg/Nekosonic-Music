<template>
  <div :class="['flex items-center gap-4 p-3 rounded-xl cursor-pointer transition group', containerClass]">
    <slot name="index" :index="index" :is-current="isCurrent">
      <span v-if="showIndex" class="text-xs text-content-3 w-6 text-right flex-shrink-0">{{ index + 1 }}</span>
    </slot>

    <div :class="['rounded-md overflow-hidden flex-shrink-0 relative', coverClass]">
      <img v-if="coverSrc" :src="coverSrc" class="w-full h-full object-cover" loading="lazy" />
      <div v-else class="w-full h-full bg-muted flex items-center justify-center">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-content-4"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      </div>
      <div v-if="isCurrent && showPlayingOverlay"
        class="absolute inset-0 bg-black/30 flex items-center justify-center">
        <div class="flex items-end gap-[2px] h-3">
          <span class="eq-bar-sm w-[2px] bg-white rounded-full" style="animation-delay: 0s"></span>
          <span class="eq-bar-sm w-[2px] bg-white rounded-full" style="animation-delay: 0.12s"></span>
          <span class="eq-bar-sm w-[2px] bg-white rounded-full" style="animation-delay: 0.24s"></span>
        </div>
      </div>
    </div>

    <div class="flex-1 min-w-0">
      <p class="text-sm font-medium truncate" :class="nameClass">{{ song.name }}</p>
      <p class="text-xs text-content-2 truncate">
        <template v-if="song.ar?.length">
          <template v-for="(a, i) in song.ar" :key="a.id || i">
            <span v-if="i > 0" class="text-content-3">/</span>
            <span class="hover:text-accent-text cursor-pointer transition" @click.stop="a.id && router.push({ name: 'artist', params: { id: a.id } })">{{ a.name }}</span>
          </template>
        </template>
        <template v-if="song.al?.name">
          <span class="text-content-3 mx-1">·</span>
          <span class="hover:text-accent-text cursor-pointer transition" @click.stop="song.al.id && router.push({ name: 'album', params: { id: song.al.id } })">{{ song.al.name }}</span>
        </template>
      </p>
    </div>

    <slot name="actions">
      <button v-if="showLike" @click.stop="player.toggleLike(song.id)" class="text-content-3 hover:text-danger transition flex-shrink-0">
        <svg v-if="player.isLiked(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="text-danger"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
      </button>
      <button v-if="showDownload" @click.stop="download.downloadSong(song)" class="text-content-3 hover:text-accent-text transition flex-shrink-0" :title="download.isDownloaded(song.id) ? '已下载' : '下载'">
        <svg v-if="download.isDownloading(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
        <svg v-else-if="download.isDownloaded(song.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent-text"><polyline points="20 6 9 17 4 12"/></svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
      </button>
      <SongItemMenu v-if="showMenu" :song-id="song.id" />
    </slot>

    <span v-if="showDuration && song.dt" class="text-xs text-content-3 flex-shrink-0">{{ formatDuration(song.dt) }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { usePlayerStore } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import { getCoverUrl, type Song } from '../utils/song';
import { formatDuration } from '../utils/format';
import SongItemMenu from './SongItemMenu.vue';

const router = useRouter();
const player = usePlayerStore();
const download = useDownload();

const props = withDefaults(defineProps<{
  song: Song;
  index: number;
  isCurrent?: boolean;
  showIndex?: boolean;
  showLike?: boolean;
  showDownload?: boolean;
  showMenu?: boolean;
  showDuration?: boolean;
  showPlayingOverlay?: boolean;
  coverSize?: string;
  coverSizeParam?: string;
  containerClass?: string;
}>(), {
  isCurrent: false,
  showIndex: false,
  showLike: false,
  showDownload: false,
  showMenu: false,
  showDuration: false,
  showPlayingOverlay: false,
  coverSize: 'w-10 h-10',
  coverSizeParam: '',
  containerClass: 'hover:bg-subtle',
});

const coverClass = computed(() => props.coverSize);
const coverSrc = computed(() => getCoverUrl(props.song, props.coverSizeParam));
const nameClass = computed(() => props.isCurrent ? 'text-accent-text' : '');
</script>
