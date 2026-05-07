<template>
  <div v-if="player.currentSong"
    class="fixed bottom-0 left-0 right-0 bg-gray-900/95 backdrop-blur border-t border-white/10 z-50 select-none">
    <!-- 歌词精简条（仅在非漫游全屏时显示） -->
    <div v-if="currentLyricText && !player.showRoamDrawer" @click="showFullLyric = !showFullLyric"
      class="px-6 py-1 text-center text-xs text-green-200/80 cursor-pointer hover:bg-white/5 transition truncate">
      {{ currentLyricText }}
    </div>

    <!-- 进度条 -->
    <div ref="progressBar" class="w-full h-1.5 bg-white/10 rounded-full relative group cursor-pointer overflow-visible"
      @mousedown.prevent="startSeek">
      <!-- 缓存进度（灰白） -->
      <div class="absolute left-0 top-0 h-full bg-white/20 rounded-full" :style="{ width: cacheProgress + '%' }"></div>
      <!-- 播放进度（绿色渐变） -->
      <div class="absolute left-0 top-0 h-full bg-gradient-to-r from-green-400 to-emerald-500 rounded-full"
        :style="{ width: displayProgress + '%' }"></div>
      <!-- 拖动圆点（基于容器定位，left 百分比） -->
      <div
        class="absolute top-1/2 -translate-y-1/2 w-3.5 h-3.5 rounded-full bg-white shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
        :style="{ left: `calc(${displayProgress}% - 7px)` }"></div>
    </div>

    <!-- 主控制区 -->
    <div class="flex items-center px-6 h-16">
      <!-- 左侧：歌曲信息 -->
      <div class="flex items-center gap-3 w-56 min-w-0">
        <img :src="player.currentSong.al?.picUrl"
          class="w-10 h-10 rounded-md object-cover flex-shrink-0 cursor-pointer hover:scale-105 transition-transform"
          @click="player.openRoamDrawer()" title="全屏展示" />
        <div class="min-w-0">
          <p class="text-sm font-medium truncate">{{ player.currentSong.name }}</p>
          <p class="text-xs text-gray-400 truncate">
            {{player.currentSong.ar?.map((a: any) => a.name).join('/')}}
          </p>
        </div>
      </div>

      <!-- 中间：控制按钮 + 时间 -->
      <div class="flex-1 flex flex-col items-center justify-center gap-1">
        <div class="flex items-center gap-5">
          <button @click="player.prev()" :disabled="player.isFmMode" :class="[
            'text-xl transition',
            player.isFmMode ? 'text-gray-600 cursor-not-allowed' : 'text-gray-400 hover:text-white',
          ]">
            ⏮
          </button>
          <button @click="player.toggle()"
            class="w-9 h-9 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 transition border border-white/20">
            <svg v-if="player.playing" width="16" height="16" viewBox="0 0 16 16" fill="currentColor"
              class="text-white">
              <rect x="3" y="2" width="3" height="12" rx="0.5" />
              <rect x="10" y="2" width="3" height="12" rx="0.5" />
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="currentColor" class="text-white">
              <path d="M4 2.5v11l9-5.5z" />
            </svg>
          </button>
          <button @click="player.next()" class="text-xl text-gray-400 hover:text-white transition">⏭</button>
        </div>
        <div class="flex items-center gap-2 text-xs text-gray-400">
          <span>{{ formatTime(player.currentTime) }}</span>
          <span>/</span>
          <span>{{ formatTime(player.duration) }}</span>
        </div>
      </div>

      <!-- 右侧：音量、模式、播放列表 -->
      <div class="w-56 flex justify-end items-center gap-2">
        <div class="flex items-center gap-1">
          <span class="text-sm text-gray-400">🔊</span>
          <div class="relative w-24 h-6 flex items-center">
            <input ref="volumeSlider" type="range" min="0" max="100" :value="volume"
              :style="{ background: volumeBarBg }" @input="handleVolumeChange"
              class="vol-slider w-full h-1.5 rounded-full appearance-none cursor-pointer bg-white/20 outline-none" />
          </div>
        </div>
        <button @click="togglePlayMode" class="text-gray-400 hover:text-white transition text-lg" :title="modeTitle">
          {{ modeIcon }}
        </button>
        <button @click="showQueuePanel = !showQueuePanel"
          class="text-gray-400 hover:text-white transition text-xl relative" title="播放列表">
          📋
          <span v-if="player.queue.length > 0"
            class="absolute -top-1 -right-1 bg-green-500 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center">
            {{ player.queue.length }}
          </span>
        </button>
      </div>
    </div>

    <!-- 队列面板 -->
    <Transition name="slide-up">
      <div v-if="showQueuePanel"
        class="border-t border-white/10 bg-gray-900/95 backdrop-blur p-4 max-h-64 overflow-y-auto">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-sm font-semibold">播放列表 ({{ player.queue.length }})</h3>
          <button @click="player.clearQueue()" class="text-xs text-red-400 hover:text-red-300 transition">清空</button>
        </div>
        <div class="space-y-1">
          <div v-for="(song, idx) in player.queue" :key="song.id + '-' + idx" @click="playFromQueue(idx)" :class="[
            'flex items-center gap-3 p-2 rounded-lg cursor-pointer transition',
            idx === player.currentIndex ? 'bg-green-500/20 text-white' : 'hover:bg-white/5 text-gray-300',
          ]">
            <span class="text-xs w-6 text-center">{{ idx + 1 }}</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-medium truncate">{{ song.name }}</p>
              <p class="text-xs text-gray-500 truncate">
                {{song.ar?.map((a: any) => a.name).join('/')}}
              </p>
            </div>
            <button @click.stop="player.removeFromQueue(idx)"
              class="text-gray-500 hover:text-red-400 transition text-sm">
              ✕
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 全屏歌词浮层 -->
    <Transition name="slide-up">
      <div v-if="showFullLyric && lyrics.length > 0 && !player.showRoamDrawer"
        class="border-t border-white/10 bg-gray-900/95 backdrop-blur p-4 max-h-72 overflow-hidden flex flex-col"
        @click.self="showFullLyric = false">
        <div class="flex justify-between mb-2">
          <h3 class="text-xs font-semibold">歌词</h3>
          <button @click="showFullLyric = false" class="text-gray-400 hover:text-white">收起</button>
        </div>
        <div ref="lyricContainer"
          class="flex-1 overflow-y-auto overflow-x-hidden whitespace-normal break-words space-y-1 text-sm text-center">
          <p v-for="(line, idx) in lyrics" :key="idx" :class="idx === currentLyricIdx
              ? 'text-green-400 font-medium scale-105 transition'
              : 'text-gray-400'
            " class="px-4 py-0.5">
            {{ line.text }}
          </p>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onBeforeUnmount, watch, onMounted } from 'vue';
import { usePlayerStore, PlayMode } from '../stores/player';
import { invoke } from '@tauri-apps/api/core';
import { useLyric } from '../composables/UserLyric';
import { listen } from '@tauri-apps/api/event';

const player = usePlayerStore();
const showQueuePanel = ref(false);
const { lyrics, currentLyricIdx, currentLyricText } = useLyric();
const showFullLyric = ref(false);
const lyricContainer = ref<HTMLElement | null>(null);
const progressBar = ref<HTMLElement | null>(null);
const isSeeking = ref(false);
const previewTime = ref(0);
const cacheProgress = ref(0);
const volume = ref(100);

let unlistenCache: (() => void) | null = null;

// 缓存进度监听
onMounted(async () => {
  const fn = await listen<number>('cache-progress', (event) => {
    cacheProgress.value = event.payload;
  });
  unlistenCache = fn;
});
onBeforeUnmount(() => {
  if (unlistenCache) unlistenCache();
});

// 播放模式
const modeTexts = { loop: '列表循环', shuffle: '随机播放', 'repeat-one': '单曲循环' };
const modeIcons = { loop: '🔁', shuffle: '🔀', 'repeat-one': '🔂' };
const modeIcon = computed(() => modeIcons[player.playMode] || '🔁');
const modeTitle = computed(() => modeTexts[player.playMode] || '列表循环');
function togglePlayMode() {
  const modes: PlayMode[] = ['loop', 'shuffle', 'repeat-one'];
  const next = modes[(modes.indexOf(player.playMode) + 1) % modes.length];
  player.setPlayMode(next);
}

// 进度条拖拽逻辑
let onDocMove: ((e: MouseEvent) => void) | null = null;
let onDocUp: (() => void) | null = null;

function startSeek(e: MouseEvent) {
  isSeeking.value = true;
  updatePreview(e);

  onDocMove = (ev: MouseEvent) => updatePreview(ev);
  onDocUp = () => finishSeek();
  document.addEventListener('mousemove', onDocMove);
  document.addEventListener('mouseup', onDocUp);
}

function updatePreview(e: MouseEvent) {
  const bar = progressBar.value;
  if (!bar || player.duration === 0) return;
  const rect = bar.getBoundingClientRect();
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  previewTime.value = ratio * player.duration;
}

function finishSeek() {
  if (!isSeeking.value) return;
  isSeeking.value = false;
  if (player.duration > 0) {
    player.seek(previewTime.value);
  }
  if (onDocMove) document.removeEventListener('mousemove', onDocMove);
  if (onDocUp) document.removeEventListener('mouseup', onDocUp);
  onDocMove = null;
  onDocUp = null;
}

onBeforeUnmount(() => {
  if (onDocMove) document.removeEventListener('mousemove', onDocMove);
  if (onDocUp) document.removeEventListener('mouseup', onDocUp);
});

const previewPercent = computed(() => {
  if (!player.duration || player.duration === 0) return 0;
  return (previewTime.value / player.duration) * 100;
});
const progressPercent = computed(() => {
  if (!player.duration || player.duration === 0) return 0;
  return (player.currentTime / player.duration) * 100;
});
const displayProgress = computed(() => {
  return isSeeking.value ? previewPercent.value : progressPercent.value;
});

function formatTime(sec: number): string {
  if (!sec || isNaN(sec)) return '0:00';
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function playFromQueue(index: number) {
  player.currentIndex = index;
  player.playCurrent();
}

async function handleVolumeChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const val = parseInt(target.value, 10);
  volume.value = val;
  await invoke('set_volume', { vol: val / 100 });
}

const volumeBarBg = computed(() => {
  const pct = volume.value;
  return `linear-gradient(to right, #34d399 0%, #10b981 ${pct}%, rgba(255,255,255,0.15) ${pct}%)`;
});

// 歌词浮层自动滚动
watch(
  () => currentLyricIdx.value,
  () => {
    if (showFullLyric.value && lyricContainer.value) {
      nextTick(() => {
        const active = lyricContainer.value?.querySelector('.text-green-400');
        active?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      });
    }
  }
);
</script>

<style scoped>
/* 样式保持不变（原有歌词浮层过渡、滑块样式等） */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.2s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

.vol-slider {
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  width: 100%;
  height: 6px;
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}

.vol-slider::-webkit-slider-runnable-track {
  height: 6px;
  border-radius: 3px;
  background: linear-gradient(to right,
      #34d399 0%,
      #10b981 var(--vol-fill),
      rgba(255, 255, 255, 0.15) var(--vol-fill));
}

.vol-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: white;
  box-shadow: 0 0 5px rgba(0, 0, 0, 0.3);
  margin-top: -3px;
  cursor: pointer;
  transition: transform 0.15s;
}

.vol-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}
</style>