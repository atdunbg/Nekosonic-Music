<template>
  <div class="flex items-center justify-between mb-4 mt-2">
    <div class="flex items-center gap-2 min-w-0">
      <component :is="icon" v-if="icon" class="w-5 h-5 text-accent-text flex-shrink-0" />
      <h2 class="text-xl font-semibold text-content truncate">{{ title }}</h2>
      <span v-if="subtitle" class="text-xs text-content-3 truncate hidden sm:inline">· {{ subtitle }}</span>
    </div>
    <button
      v-if="showMore"
      @click="$emit('more')"
      class="flex items-center gap-1 text-sm text-content-2 hover:text-content transition flex-shrink-0"
    >
      <span>更多</span>
      <IconChevronRight class="w-4 h-4" />
    </button>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue';
import IconChevronRight from '~icons/lucide/chevron-right';

defineProps<{
  title: string;
  subtitle?: string;
  icon?: Component;
  showMore?: boolean;
}>();

defineEmits<{
  (e: 'more'): void;
}>();
</script>
