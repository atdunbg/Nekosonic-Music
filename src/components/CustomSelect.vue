<template>
  <div class="relative" ref="container">
    <button
      @click="toggle"
      class="flex items-center justify-between bg-subtle border border-line rounded-lg px-3 py-1.5 text-sm text-content outline-none transition min-w-[140px] hover:border-content-3 focus:border-accent focus:shadow-[0_0_0_2px_var(--c-accent-dim)]"
      :class="{ 'border-accent shadow-[0_0_0_2px_var(--c-accent-dim)]': isOpen }"
    >
      <span>{{ currentLabel }}</span>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="transition-transform flex-shrink-0 ml-2" :class="{ 'rotate-180': isOpen }"><polyline points="6 9 12 15 18 9"/></svg>
    </button>
    <Transition name="dropdown">
      <div v-if="isOpen" class="absolute right-0 top-full mt-1 bg-surface border border-line rounded-lg shadow-xl z-50 py-1 min-w-full overflow-hidden">
        <button
          v-for="(label, key) in options"
          :key="key"
          @click="select(key)"
          class="w-full text-left px-3 py-2 text-sm transition flex items-center justify-between"
          :class="modelValue === key ? 'bg-accent-dim text-accent-text' : 'text-content-2 hover:bg-subtle hover:text-content'"
        >
          <span>{{ label }}</span>
          <svg v-if="modelValue === key" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';

const props = defineProps<{
  modelValue: string;
  options: Record<string, string>;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const isOpen = ref(false);
const container = ref<HTMLElement | null>(null);

const currentLabel = computed(() => props.options[props.modelValue] || '');

function toggle() {
  isOpen.value = !isOpen.value;
}

function select(key: string) {
  emit('update:modelValue', key);
  isOpen.value = false;
}

function onClickOutside(e: MouseEvent) {
  if (container.value && !container.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', onClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', onClickOutside);
});
</script>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
