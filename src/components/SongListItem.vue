<template>
  <div :class="['flex items-center gap-4 p-3 rounded-xl cursor-pointer transition group', containerClass]">
    <slot name="index" :index="index" :is-current="isCurrent">
      <div v-if="showIndex" class="w-6 text-right flex-shrink-0 flex items-center justify-end h-5">
        <div v-if="isCurrent && showPlayingOverlay" class="flex items-center justify-end">
          <div class="flex items-center gap-[3px] h-4">
            <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 50%; animation-delay: 0ms"></span>
            <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 100%; animation-delay: 150ms"></span>
            <span class="w-[3px] bg-accent-text rounded-full animate-bounce" style="height: 35%; animation-delay: 300ms"></span>
          </div>
        </div>
        <template v-else>
          <span class="text-xs text-content-3 group-hover:hidden">{{ index + 1 }}</span>
          <IconPlay class="hidden group-hover:block text-content" style="font-size: 14px" />
        </template>
      </div>
    </slot>

    <div :class="['rounded-md overflow-hidden flex-shrink-0 relative', coverClass]">
      <img v-if="coverSrc" :src="coverSrc" class="w-full h-full object-cover" loading="lazy" />
      <div v-else class="w-full h-full bg-muted flex items-center justify-center">
        <IconMusic style="font-size: 14px" class="text-content-4" />
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
        <IconHeart v-if="player.isLiked(song.id)" class="w-4 h-4 text-danger [&>path]:fill-current [&>path]:stroke-0" />
        <IconHeart v-else class="w-4 h-4" />
      </button>
      <button v-if="showDownload" @click.stop="download.downloadSong(song)" class="text-content-3 hover:text-accent-text transition flex-shrink-0" :title="download.isDownloaded(song.id) ? '已下载' : '下载'">
        <IconLoader2 v-if="download.isDownloading(song.id)" class="w-4 h-4 animate-spin" />
        <IconCheck v-else-if="download.isDownloaded(song.id)" class="w-4 h-4 text-accent-text" />
        <IconDownload v-else class="w-4 h-4" />
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
import IconPlay from '~icons/lucide/play';
import IconMusic from '~icons/lucide/music';
import IconHeart from '~icons/lucide/heart';
import IconLoader2 from '~icons/lucide/loader-2';
import IconCheck from '~icons/lucide/check';
import IconDownload from '~icons/lucide/download';

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
