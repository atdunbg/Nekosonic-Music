<template>
  <div
    class="group relative rounded-xl overflow-hidden bg-subtle hover:bg-muted transition cursor-pointer"
    @click="$emit('click', playlist)"
  >
    <!-- 封面 -->
    <div class="relative aspect-square overflow-hidden">
      <img
        v-if="coverUrl"
        :src="coverUrl"
        :alt="playlist.name"
        class="w-full h-full object-cover transition duration-300 group-hover:scale-105 group-hover:brightness-90"
        loading="lazy"
      />
      <div v-else class="w-full h-full bg-muted flex items-center justify-center">
        <IconMusic class="w-8 h-8 text-content-4" />
      </div>

      <!-- 顶部渐变 + 播放量 -->
      <div
        v-if="playCount"
        class="absolute top-0 left-0 right-0 h-1/3 bg-gradient-to-b from-black/40 to-transparent flex items-start justify-end p-2"
      >
        <div class="flex items-center gap-1 text-white text-xs">
          <IconPlay class="w-3 h-3 fill-current" />
          <span>{{ formattedPlayCount }}</span>
        </div>
      </div>

      <!-- 悬浮播放按钮 -->
      <button
        v-if="showPlayBtn"
        @click.stop="$emit('play', playlist)"
        class="absolute bottom-2 right-2 w-9 h-9 flex items-center justify-center rounded-full bg-white/20 backdrop-blur-md text-white opacity-0 translate-y-2 group-hover:opacity-100 group-hover:translate-y-0 hover:bg-white/30 transition"
        :title="isCurrentPlaying ? '暂停' : '播放'"
      >
        <IconPause v-if="isCurrentPlaying" class="w-4 h-4 fill-current" />
        <IconPlay v-else class="w-4 h-4 fill-current ml-0.5" />
      </button>
    </div>

    <!-- 信息 -->
    <div class="p-3">
      <p class="text-sm font-medium truncate text-content">{{ playlist.name }}</p>
      <p v-if="subtitle" class="text-xs text-content-2 mt-1 truncate">{{ subtitle }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import IconMusic from '~icons/lucide/music';
import IconPlay from '~icons/lucide/play';
import IconPause from '~icons/lucide/pause';
import { formatPlayCount } from '../../utils/format';

export interface PlaylistCardData {
  id: number;
  name: string;
  picUrl?: string;
  coverImgUrl?: string;
  playCount?: number;
  copywriter?: string;
  description?: string;
  creator?: { name?: string } | string;
}

const props = withDefaults(defineProps<{
  playlist: PlaylistCardData;
  showPlayBtn?: boolean;
  coverSize?: string;
  isCurrentPlaying?: boolean;
}>(), {
  showPlayBtn: true,
  isCurrentPlaying: false,
});

defineEmits<{
  (e: 'click', playlist: PlaylistCardData): void;
  (e: 'play', playlist: PlaylistCardData): void;
}>();

const coverUrl = computed(() => props.playlist.picUrl || props.playlist.coverImgUrl || '');
const playCount = computed(() => props.playlist.playCount || 0);
const formattedPlayCount = computed(() => formatPlayCount(playCount.value));
const subtitle = computed(() => {
  const c = props.playlist;
  if (c.copywriter) return c.copywriter;
  if (c.description) return c.description;
  const creator = c.creator;
  if (typeof creator === 'string') return creator;
  if (creator?.name) return `by ${creator.name}`;
  return '';
});
</script>
