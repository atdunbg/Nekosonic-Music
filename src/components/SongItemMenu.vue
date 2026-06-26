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
      <button @click.stop="handleShare" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-content-2 hover:bg-subtle hover:text-content transition">
        <IconShare2 style="font-size: 14px" />
        分享
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount, onMounted } from 'vue';
import { useUiStore } from '../stores/ui';
import { showToast } from '../composables/useToast';
import IconEllipsis from '~icons/lucide/ellipsis';
import IconMessageSquare from '~icons/lucide/message-square';
import IconShare2 from '~icons/lucide/share-2';

const ui = useUiStore();
const props = defineProps<{ songId: number }>();
const open = ref(false);
const menuRef = ref<HTMLElement | null>(null);

function toggle() {
  open.value = !open.value;
}

function handleComment() {
  open.value = false;
  ui.openCommentForSong(props.songId);
}

async function handleShare() {
  open.value = false;
  const url = `https://music.163.com/song?id=${props.songId}`;
  try {
    await navigator.clipboard.writeText(url);
    showToast('链接已复制', 'success');
  } catch {
    showToast('复制失败', 'error');
  }
}

function onClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(() => document.addEventListener('click', onClickOutside));
onBeforeUnmount(() => document.removeEventListener('click', onClickOutside));
</script>
