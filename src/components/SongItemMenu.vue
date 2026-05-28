<template>
  <div class="relative flex-shrink-0" ref="menuRef">
    <button @click.stop="toggle" class="text-content-3 hover:text-content transition p-1 rounded-md hover:bg-subtle">
      <IconEllipsis class="w-4 h-4 fill-current" />
    </button>
    <div v-if="open"
      class="absolute right-0 top-full mt-1 bg-surface border border-line rounded-xl shadow-xl z-50 py-1 min-w-[120px]">
      <button @click.stop="handleComment" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-content-2 hover:bg-subtle hover:text-content transition">
        <IconMessageSquare style="font-size: 14px" />
        评论
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount, onMounted } from 'vue';
import { usePlayerStore } from '../stores/player';
import IconEllipsis from '~icons/lucide/ellipsis';
import IconMessageSquare from '~icons/lucide/message-square';

const player = usePlayerStore();
const props = defineProps<{ songId: number }>();
const open = ref(false);
const menuRef = ref<HTMLElement | null>(null);

function toggle() {
  open.value = !open.value;
}

function handleComment() {
  open.value = false;
  player.openCommentForSong(props.songId);
}

function onClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => document.addEventListener('click', onClickOutside));
onBeforeUnmount(() => document.removeEventListener('click', onClickOutside));
</script>
