<template>
  <div
    class="group flex flex-col items-center cursor-pointer transition"
    @click="$emit('click', artist)"
  >
    <!-- 圆形封面 -->
    <div class="relative w-full aspect-square rounded-full overflow-hidden mb-3">
      <img
        v-if="coverUrl"
        :src="coverUrl"
        :alt="artist.name"
        class="w-full h-full object-cover transition duration-300 group-hover:scale-110 group-hover:brightness-90"
        loading="lazy"
      />
      <div v-else class="w-full h-full bg-muted flex items-center justify-center">
        <IconUser class="w-8 h-8 text-content-4" />
      </div>
      <!-- 悬浮遮罩 + 图标 -->
      <div class="absolute inset-0 bg-black/30 opacity-0 group-hover:opacity-100 transition flex items-center justify-center">
        <IconArtist class="w-7 h-7 text-white" />
      </div>
    </div>

    <!-- 信息 -->
    <p class="text-sm font-medium truncate text-content text-center w-full">{{ artist.name }}</p>
    <p v-if="subtitle" class="text-xs text-content-3 mt-0.5 text-center truncate w-full">{{ subtitle }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import IconUser from '~icons/lucide/user';
import IconArtist from '~icons/lucide/mic';

export interface ArtistCardData {
  id: number;
  name: string;
  picUrl?: string;
  img1v1Url?: string;
  musicSize?: number;
  albumSize?: number;
}

const props = defineProps<{
  artist: ArtistCardData;
}>();

defineEmits<{
  (e: 'click', artist: ArtistCardData): void;
}>();

const coverUrl = computed(() => props.artist.picUrl || props.artist.img1v1Url || '');
const subtitle = computed(() => {
  const a = props.artist;
  const parts: string[] = [];
  if (a.musicSize) parts.push(`${a.musicSize}首`);
  if (a.albumSize) parts.push(`${a.albumSize}张专辑`);
  return parts.join(' · ');
});
</script>
