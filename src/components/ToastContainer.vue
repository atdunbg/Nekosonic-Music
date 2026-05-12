<template>
  <TransitionGroup name="toast" tag="div" class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      @click="dismiss(toast.id)"
      class="pointer-events-auto min-w-[280px] max-w-[400px] px-4 py-3 rounded-lg shadow-lg bg-surface/95 backdrop-blur cursor-pointer border-l-4"
      :class="borderClass(toast.type)"
    >
      <p class="text-sm font-medium" :class="textClass(toast.type)">{{ toast.message }}</p>
    </div>
  </TransitionGroup>
</template>

<script setup lang="ts">
import { useToast, type Toast } from '../composables/useToast';

const { toasts } = useToast();

function borderClass(type: Toast['type']) {
  return {
    success: 'border-accent',
    error: 'border-danger',
    info: 'border-info',
  }[type];
}

function textClass(type: Toast['type']) {
  return {
    success: 'text-accent-text',
    error: 'text-danger',
    info: 'text-info',
  }[type];
}

function dismiss(id: number) {
  const idx = toasts.value.findIndex(t => t.id === id);
  if (idx !== -1) toasts.value.splice(idx, 1);
}
</script>

<style scoped>
.toast-enter-active {
  transition: all 0.3s ease-out;
}
.toast-leave-active {
  transition: all 0.2s ease-in;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
.toast-move {
  transition: transform 0.3s ease;
}
</style>
