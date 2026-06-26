<template>
  <div class="grid gap-4" :class="gridClass">
    <template v-if="loading && !items.length">
      <div
        v-for="i in skeletonCount"
        :key="'skel-' + i"
        class="rounded-xl overflow-hidden bg-subtle animate-pulse"
      >
        <div class="w-full aspect-square bg-muted"></div>
        <div class="p-3 space-y-2">
          <div class="h-4 bg-muted rounded w-3/4"></div>
          <div class="h-3 bg-muted rounded w-1/2"></div>
        </div>
      </div>
    </template>

    <template v-else-if="error && !items.length">
      <div class="col-span-full flex flex-col items-center justify-center py-12 gap-3">
        <p class="text-content-2 text-sm">{{ errorText }}</p>
        <button
          @click="$emit('retry')"
          class="px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition"
        >
          重试
        </button>
      </div>
    </template>

    <template v-else>
      <slot />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  items: unknown[];
  loading?: boolean;
  error?: boolean;
  errorText?: string;
  skeletonCount?: number;
  /** 自定义栅格列数类名，默认响应式 */
  gridClass?: string;
}>(), {
  loading: false,
  error: false,
  errorText: '加载失败',
  skeletonCount: 6,
  gridClass: '',
});

const gridClass = computed(() => props.gridClass || 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6');

defineEmits<{
  (e: 'retry'): void;
}>();
</script>
