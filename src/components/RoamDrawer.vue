<template>
  <Transition name="drawer">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex flex-col backdrop-blur-xl"
      :class="!player.dominantColor && 'bg-surface/95'"
      :style="player.dominantColor ? { backgroundColor: player.dominantColor } : {}"
    >
      <div v-if="player.dominantColor" class="absolute inset-0 bg-black/60 pointer-events-none"></div>
      <TitleBar :dark-mode="!!player.dominantColor" @close="player.closeRoamDrawer()">
        <template #left>
          <button @click="player.closeRoamDrawer()" :class="player.dominantColor ? 'text-white/60 hover:text-white' : 'text-content-2 hover:text-content'" class="transition">
            <IconChevronDown class="w-5 h-5" />
          </button>
        </template>
      </TitleBar>
      <div class="flex-1 min-h-0 flex px-8 pb-8 gap-0 relative z-10">
        <div class="w-2/5 flex flex-col items-center justify-center flex-shrink-0">
          <img
            v-if="roamCoverUrl && !roamCoverError"
            :src="roamCoverUrl"
            class="w-72 h-72 rounded-3xl object-cover shadow-2xl mb-4"
            @error="roamCoverError = true"
          />
          <div
            v-else
            class="w-72 h-72 rounded-3xl flex items-center justify-center shadow-2xl mb-4"
            :class="player.dominantColor ? 'bg-white/10' : 'bg-muted'"
          >
            <IconMusic class="w-16 h-16" :class="player.dominantColor ? 'text-white/30' : 'text-content-4'" />
          </div>
          <h1 class="text-2xl font-bold text-center" :class="player.dominantColor ? 'text-white' : 'text-content'">{{ roamSong?.name }}</h1>
          <p class="mt-2 text-center" :class="player.dominantColor ? 'text-white/70' : 'text-content-2'">
            <template v-for="(a, i) in roamSong?.ar || []" :key="a.id || i">
              <span v-if="i > 0" :class="player.dominantColor ? 'text-white/40' : 'text-content-3'">/</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click="a.id && navigateFromDrawer({ name: 'artist', params: { id: a.id } })">{{ a.name }}</span>
            </template>
            <template v-if="roamSong?.al?.name">
              <span :class="player.dominantColor ? 'text-white/40' : 'text-content-3'" class="mx-1">·</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click="roamSong!.al.id && navigateFromDrawer({ name: 'album', params: { id: roamSong!.al.id } })">{{ roamSong.al.name }}</span>
            </template>
          </p>
        </div>
        <div class="w-3/5 relative min-h-0 overflow-hidden flex flex-col">
          <div class="flex items-center gap-1 mb-3 px-4">
            <button @click="player.roamTab = 'lyric'"
              class="px-3 py-1 rounded-full text-sm transition"
              :class="player.dominantColor
                ? (player.roamTab === 'lyric' ? 'bg-white/15 text-white font-medium' : 'text-white/50 hover:text-white/80')
                : (player.roamTab === 'lyric' ? 'bg-muted text-content font-medium' : 'text-content-3 hover:text-content')">
              歌词
            </button>
            <button @click="player.roamTab = 'comment'"
              class="px-3 py-1 rounded-full text-sm transition"
              :class="player.dominantColor
                ? (player.roamTab === 'comment' ? 'bg-white/15 text-white font-medium' : 'text-white/50 hover:text-white/80')
                : (player.roamTab === 'comment' ? 'bg-muted text-content font-medium' : 'text-content-3 hover:text-content')">
              评论
            </button>
            <button v-if="hasTranslation" @click="toggleTranslation"
              class="ml-auto px-2.5 py-1 rounded-full text-xs transition flex items-center gap-1"
              :class="player.dominantColor
                ? (showTranslation ? 'bg-white/15 text-white font-medium' : 'text-white/40 hover:text-white/70')
                : (showTranslation ? 'bg-muted text-content font-medium' : 'text-content-4 hover:text-content-2')">
              <IconLanguages class="w-3 h-3" />
              译
            </button>
          </div>
          <div v-show="player.roamTab === 'lyric'" ref="lyricScrollContainer" class="flex-1 min-h-0 overflow-y-auto custom-scroll px-4">
            <div v-if="lyrics.length > 0" class="w-full max-w-lg mx-auto text-center"
              :style="{ paddingTop: roamLyricPadPx + 'px', paddingBottom: roamLyricPadPx + 'px' }">
              <p
                v-for="(line, idx) in lyrics"
                :key="idx"
                :class="getRoamLyricClass(idx)"
                class="roam-lyric-line px-4 py-3 rounded-lg cursor-pointer whitespace-nowrap transition-[font-size] duration-300 ease-out"
                @click="seekToRoamLyric(line.time)"
                @mouseenter="roamLyricHovering = true"
                @mouseleave="roamLyricHovering = false"
              >
                {{ line.text }}
                <span v-if="showTranslation && line.translation" class="block text-sm mt-1" :class="getTranslationClass(idx)">{{ line.translation }}</span>
              </p>
            </div>
            <div v-else :class="player.dominantColor ? 'text-white/40' : 'text-content-3'" class="text-center mt-8">暂无歌词</div>
          </div>
          <div v-show="player.roamTab === 'comment'" class="flex-1 min-h-0 overflow-y-auto px-4 pb-4">
            <CommentSection v-if="roamSong" :type="0" :id="player.commentSongId || roamSong.id" :key="player.commentSongId || roamSong.id" :dark-mode="!!player.dominantColor" />
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount, computed, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { usePlayerStore } from '../stores/player';
import { getCoverUrl, extractDominantColor } from '../utils/song';
import { useLyric } from '../composables/UserLyric';
import TitleBar from './TitleBar.vue';
import CommentSection from './CommentSection.vue';
import IconChevronDown from '~icons/lucide/chevron-down';
import IconMusic from '~icons/lucide/music';
import IconLanguages from '~icons/lucide/languages';

defineProps<{
  visible: boolean;
}>();

const router = useRouter();
const player = usePlayerStore();

const { lyrics, currentLyricIdx, hasTranslation, showTranslation, toggleTranslation } = useLyric();
const lyricScrollContainer = ref<HTMLElement | null>(null);
const roamLyricHovering = ref(false);
const roamLyricPadPx = ref(0);
const roamSong = computed(() => player.currentSong);
const roamCoverError = ref(false);
const roamCoverUrl = computed(() => {
  if (!roamSong.value) return '';
  return getCoverUrl(roamSong.value) || '';
});

watch(roamCoverUrl, async (url) => {
  roamCoverError.value = false;
  if (url) {
    const color = await extractDominantColor(url);
    player.dominantColor = color;
  } else {
    player.dominantColor = '';
  }
});

let roamResizeObserver: ResizeObserver | null = null;

function updateRoamLyricPad() {
  if (lyricScrollContainer.value) {
    roamLyricPadPx.value = Math.floor(lyricScrollContainer.value.clientHeight / 2);
  }
}

watch(() => player.showRoamDrawer, (val) => {
  if (val) {
    nextTick(() => {
      updateRoamLyricPad();
      if (roamResizeObserver) roamResizeObserver.disconnect();
      if (lyricScrollContainer.value) {
        roamResizeObserver = new ResizeObserver(() => updateRoamLyricPad());
        roamResizeObserver.observe(lyricScrollContainer.value);
      }
      scrollToRoamActiveLyric();
    });
  } else {
    if (roamResizeObserver) {
      roamResizeObserver.disconnect();
      roamResizeObserver = null;
    }
  }
});

onBeforeUnmount(() => {
  if (roamResizeObserver) {
    roamResizeObserver.disconnect();
    roamResizeObserver = null;
  }
});

watch(currentLyricIdx, () => {
  if (player.showRoamDrawer && !roamLyricHovering.value) {
    nextTick(() => scrollToRoamActiveLyric());
  }
});

watch(showTranslation, () => {
  if (player.showRoamDrawer && !roamLyricHovering.value) {
    nextTick(() => scrollToRoamActiveLyric());
  }
});

function scrollToRoamActiveLyric() {
  if (!lyricScrollContainer.value || roamLyricHovering.value) return;
  const container = lyricScrollContainer.value;
  const active = container.querySelector('.roam-lyric-active') as HTMLElement | null;
  if (!active) return;
  const target = active.offsetTop - container.clientHeight / 2 + active.clientHeight / 2;
  const start = container.scrollTop;
  const distance = target - start;
  if (Math.abs(distance) < 1) return;
  const duration = 400;
  const startTime = performance.now();
  function animate(now: number) {
    const elapsed = now - startTime;
    const progress = Math.min(elapsed / duration, 1);
    const ease = 1 - Math.pow(1 - progress, 3);
    container.scrollTop = start + distance * ease;
    if (progress < 1) requestAnimationFrame(animate);
  }
  requestAnimationFrame(animate);
}

function getTranslationClass(idx: number): string {
  const diff = Math.abs(idx - currentLyricIdx.value);
  const hasColor = !!player.dominantColor;
  if (idx === currentLyricIdx.value) return hasColor ? 'text-[var(--c-accent)]' : 'text-accent-text';
  if (diff === 1) return hasColor ? 'text-white/70' : 'text-content/70';
  if (diff === 2) return hasColor ? 'text-white/50' : 'text-content-2/50';
  return hasColor ? 'text-white/35' : 'text-content-3/35';
}

function getRoamLyricClass(idx: number): string {
  const diff = Math.abs(idx - currentLyricIdx.value);
  const hasColor = !!player.dominantColor;
  if (idx === currentLyricIdx.value) {
    return hasColor
      ? 'roam-lyric-active font-bold text-xl text-[var(--c-accent)]'
      : 'roam-lyric-active text-accent-text font-semibold text-xl';
  }
  if (diff === 1) return hasColor ? 'text-white/70 text-lg' : 'text-content/70 text-lg';
  if (diff === 2) return hasColor ? 'text-white/50 text-[1rem]' : 'text-content-2/50 text-[1rem]';
  return hasColor ? 'text-white/35 text-[1rem]' : 'text-content-3/35 text-[1rem]';
}

function seekToRoamLyric(time: number) {
  if (time != null && player.duration > 0) {
    player.seek(time);
  }
}

function navigateFromDrawer(routeLocation: { name: string; params: any }) {
  player.closeRoamDrawer();
  router.push(routeLocation);
}
</script>

<style scoped>
.drawer-enter-active,
.drawer-leave-active { transition: transform 0.3s ease; }
.drawer-enter-from,
.drawer-leave-to { transform: translateY(100%); }
.custom-scroll::-webkit-scrollbar { width: 0; display: none; }
</style>
