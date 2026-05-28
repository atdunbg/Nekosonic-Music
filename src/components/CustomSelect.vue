<template>
  <div class="relative" ref="container">
    <button
      @click="toggle"
      class="flex items-center justify-between bg-subtle border border-line rounded-lg px-3 py-1.5 text-sm text-content outline-none transition min-w-[140px] max-w-[320px] hover:border-content-3 focus:border-accent focus:shadow-[0_0_0_2px_var(--c-accent-dim)]"
      :class="{ 'border-accent shadow-[0_0_0_2px_var(--c-accent-dim)]': isOpen }"
    >
      <span class="truncate">{{ currentLabel }}</span>
      <IconChevronDown style="font-size: 12px" class="transition-transform flex-shrink-0 ml-2" :class="{ 'rotate-180': isOpen }" />
    </button>
    <Transition name="dropdown">
      <div v-if="isOpen" class="absolute right-0 top-full mt-1 bg-surface border border-line rounded-lg shadow-xl z-50 py-1 min-w-full max-w-[360px] overflow-hidden">
        <button
          v-for="(label, key) in options"
          :key="key"
          @click="select(key)"
          class="w-full text-left px-3 py-2 text-sm transition flex items-center justify-between gap-2"
          :class="modelValue === key ? 'bg-accent-dim text-accent-text' : 'text-content-2 hover:bg-subtle hover:text-content'"
        >
          <span class="truncate">{{ label }}</span>
          <IconCheck v-if="modelValue === key" style="font-size: 14px" />
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import IconChevronDown from '~icons/lucide/chevron-down';
import IconCheck from '~icons/lucide/check';

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
