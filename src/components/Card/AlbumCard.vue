<template>
  <div
    class="group relative rounded-xl overflow-hidden bg-subtle hover:bg-muted transition cursor-pointer"
    @click="$emit('click', album)"
  >
    <!-- 封面 -->
    <div class="relative aspect-square overflow-hidden">
      <img
        v-if="coverUrl"
        :src="coverUrl"
        :alt="album.name"
        class="w-full h-full object-cover transition duration-300 group-hover:scale-105 group-hover:brightness-90"
        loading="lazy"
      />
      <div v-else class="w-full h-full bg-muted flex items-center justify-center">
        <IconDisc class="w-8 h-8 text-content-4" />
      </div>

      <!-- 悬浮播放按钮 -->
              <button
                v-if="showPlayBtn"
                @click.stop="$emit('play', album)"
                class="absolute bottom-2 right-2 w-9 h-9 flex items-center justify-center rounded-full bg-white/20 backdrop-blur-md text-white opacity-0 translate-y-2 group-hover:opacity-100 group-hover:translate-y-0 hover:bg-white/30 transition"
                :title="isCurrentPlaying ? '暂停' : '播放'"
              >
                <IconPause v-if="isCurrentPlaying" class="w-4 h-4 fill-current" />
                <IconPlay v-else class="w-4 h-4 fill-current ml-0.5" />
              </button>
    </div>

    <!-- 信息 -->
    <div class="p-3">
      <p class="text-sm font-medium truncate text-content">{{ album.name }}</p>
      <p v-if="subtitle" class="text-xs text-content-2 mt-1 truncate">{{ subtitle }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import IconDisc from '~icons/lucide/disc-3';
import IconPlay from '~icons/lucide/play';
import IconPause from '~icons/lucide/pause';
import { formatDate } from '../../utils/format';

export interface AlbumCardData {
  id: number;
  name: string;
  picUrl?: string;
  blurPicUrl?: string;
  artist?: { name?: string } | string;
  artists?: { name?: string }[];
  publishTime?: number;
  size?: number;
}

const props = withDefaults(defineProps<{
  album: AlbumCardData;
  showPlayBtn?: boolean;
  isCurrentPlaying?: boolean;
}>(), {
  showPlayBtn: true,
  isCurrentPlaying: false,
});

defineEmits<{
  (e: 'click', album: AlbumCardData): void;
  (e: 'play', album: AlbumCardData): void;
}>();

const coverUrl = computed(() => props.album.picUrl || props.album.blurPicUrl || '');
const subtitle = computed(() => {
  const a = props.album;
  const artistName = typeof a.artist === 'string'
    ? a.artist
    : a.artist?.name || a.artists?.map(ar => ar.name).filter(Boolean).join(' / ') || '';
  const date = a.publishTime ? formatDate(a.publishTime) : '';
  const count = a.size ? `${a.size}首` : '';
  const parts = [artistName, date, count].filter(Boolean);
  return parts.join(' · ');
});
</script>
