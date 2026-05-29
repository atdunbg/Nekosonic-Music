<template>
  <Transition name="fade">
    <div v-if="visible" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="$emit('cancel')">
      <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-6 select-auto">
        <h2 class="text-lg font-semibold text-content mb-1">关闭确认</h2>
        <p class="text-sm text-content-2 mb-5">你希望如何处理？</p>
        <div class="space-y-2.5 mb-4">
          <button @click="handleAction('minimize')"
            class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-subtle hover:bg-muted transition text-left">
            <div class="w-9 h-9 rounded-lg bg-accent-dim flex items-center justify-center flex-shrink-0">
              <IconMaximize2 class="w-[18px] h-[18px] text-accent-text" />
            </div>
            <div>
              <p class="text-sm font-medium text-content">最小化到托盘</p>
              <p class="text-xs text-content-3">程序继续在后台运行</p>
            </div>
          </button>
          <button @click="handleAction('exit')"
            class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-subtle hover:bg-muted transition text-left">
            <div class="w-9 h-9 rounded-lg bg-danger-dim flex items-center justify-center flex-shrink-0">
              <IconX class="w-[18px] h-[18px] text-danger" />
            </div>
            <div>
              <p class="text-sm font-medium text-content">退出程序</p>
              <p class="text-xs text-content-3">完全关闭应用程序</p>
            </div>
          </button>
        </div>
        <label class="flex items-center gap-2 cursor-pointer mb-4 select-none">
          <input type="checkbox" v-model="dontAskAgain" />
          <span class="text-xs text-content-2">不再询问，记住我的选择</span>
        </label>
        <button @click="$emit('cancel')"
          class="w-full py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">
          取消
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { CloseAction } from '../stores/settings';
import IconMaximize2 from '~icons/lucide/maximize-2';
import IconX from '~icons/lucide/x';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  confirm: [action: CloseAction, remember: boolean];
  cancel: [];
}>();

const dontAskAgain = ref(false);

function handleAction(action: CloseAction) {
  emit('confirm', action, dontAskAgain.value);
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from,
.fade-leave-to { opacity: 0; }
</style>
