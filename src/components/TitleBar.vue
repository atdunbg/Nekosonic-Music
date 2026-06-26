<template>
  <div
    data-tauri-drag-region
    class="h-10 flex items-center justify-between px-4 flex-shrink-0 select-none relative z-10"
    :style="titleBarBgStyle"
  >
    <slot name="left">
      <span v-if="!darkMode" class="text-xs text-content-3 font-medium ml-2">Nekosonic Music</span>
    </slot>
    <div class="flex items-center gap-1.5">
      <button @click="minimizeWindow" class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition" title="最小化"></button>
      <button @click="toggleMaximize" class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition" title="最大化/还原"></button>
      <button @click="$emit('close')" class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition" title="关闭"></button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useSettingsStore } from '../stores/settings';
import { minimizeWindow, toggleMaximize } from '../composables/useWindowControls';

const props = defineProps<{
  darkMode?: boolean;
  transparent?: boolean;
}>();

defineEmits<{
  close: [];
}>();

const settings = useSettingsStore();

const titleBarBgStyle = computed(() => {
  if (props.transparent) return {};
  if (settings.currentWallpaper.path) return {}; // 有壁纸时透明，由遮罩层统一提供背景
  if (props.darkMode) return {};
  return { backgroundColor: settings.currentColors.surface };
});
</script>
