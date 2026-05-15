<template>
  <div
    class="fixed bottom-0 left-0 right-0 bg-surface/95 backdrop-blur border-t border-line z-50 select-none">

    <div ref="progressBar" class="w-full h-1.5 bg-muted rounded-full relative group cursor-pointer overflow-visible"
      @mousedown.prevent="startSeek">
      <div class="absolute left-0 top-0 h-full bg-emphasis rounded-full" :style="{ width: cacheProgress + '%' }"></div>
      <div class="absolute left-0 top-0 h-full bg-accent rounded-full"
        :style="{ width: displayProgress + '%' }"></div>
      <div
        class="absolute top-1/2 -translate-y-1/2 w-3.5 h-3.5 rounded-full bg-white shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
        :style="{ left: `calc(${displayProgress}% - 7px)` }"></div>
    </div>

    <div class="flex items-center px-6 h-16">
      <div class="flex items-center gap-3 w-56 min-w-0">
        <div v-if="player.currentSong?.al?.picUrl" class="w-10 h-10 rounded-md overflow-hidden flex-shrink-0 cursor-pointer hover:scale-105 transition-transform" @click="player.toggleRoamDrawer()" title="全屏展示">
          <img :src="player.currentSong.al.picUrl" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-10 h-10 rounded-md flex-shrink-0 bg-muted flex items-center justify-center cursor-pointer hover:scale-105 transition-transform" @click="player.toggleRoamDrawer()" title="全屏展示">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-content-3"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium truncate">{{ player.currentSong?.name }}</p>
          <p class="text-xs text-content-2 truncate">
            <template v-for="(a, i) in player.currentSong?.ar || []" :key="a.id || i">
              <span v-if="i > 0" class="text-content-3">/</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click.stop="a.id && router.push({ name: 'artist', params: { id: a.id } })">{{ a.name }}</span>
            </template>
            <template v-if="player.currentSong?.al?.name">
              <span class="text-content-3 mx-1">·</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click.stop="player.currentSong!.al.id && router.push({ name: 'album', params: { id: player.currentSong!.al.id } })">{{ player.currentSong.al.name }}</span>
            </template>
          </p>
        </div>
        <button @click="player.currentSong && player.toggleLike(player.currentSong.id)" class="flex-shrink-0 transition" :class="player.currentSong && player.isLiked(player.currentSong.id) ? 'text-danger' : 'text-content-3 hover:text-danger'">
          <svg v-if="player.currentSong && player.isLiked(player.currentSong.id)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
        </button>
        <button v-if="player.currentSong" @click="player.openRoamDrawer('comment')" class="flex-shrink-0 text-content-3 hover:text-accent-text transition" title="评论">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
        </button>
        <button v-if="player.currentSong && !download.isDownloaded(player.currentSong!.id) && !download.isDownloading(player.currentSong!.id)" @click="download.downloadSong(player.currentSong)" class="flex-shrink-0 text-content-3 hover:text-accent-text transition" title="下载">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </button>
        <svg v-if="player.currentSong && download.isDownloading(player.currentSong!.id)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="flex-shrink-0 animate-spin text-content-3"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
      </div>

      <div class="flex-1 flex flex-col items-center justify-center gap-1">
        <div class="flex items-center gap-5">
          <button @click="player.prev()" :disabled="player.isFmMode" :class="[
            'transition',
            player.isFmMode ? 'text-content-4 cursor-not-allowed' : 'text-content-2 hover:text-content',
          ]">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5"/></svg>
          </button>
          <button @click="player.toggle()"
            class="w-9 h-9 flex items-center justify-center rounded-full bg-muted hover:bg-emphasis transition border border-emphasis">
            <svg v-if="player.playing" width="16" height="16" viewBox="0 0 16 16" fill="currentColor"
              class="text-white">
              <rect x="3" y="2" width="3" height="12" rx="0.5" />
              <rect x="10" y="2" width="3" height="12" rx="0.5" />
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="currentColor" class="text-white">
              <path d="M4 2.5v11l9-5.5z" />
            </svg>
          </button>
          <button @click="player.next()" class="text-content-2 hover:text-content transition">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19"/></svg>
          </button>
        </div>
        <div class="flex items-center gap-2 text-xs text-content-2">
          <span>{{ formatTime(player.currentTime) }}</span>
          <span>/</span>
          <span>{{ formatTime(player.duration) }}</span>
        </div>
      </div>

      <div class="w-56 flex justify-end items-center gap-2">
        <div class="flex items-center gap-1">
          <button @click="toggleMute" class="text-content-2 hover:text-content transition">
            <svg v-if="player.volume === 0" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>
            <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07"/></svg>
          </button>
          <div class="relative w-20 h-6 flex items-center">
            <input ref="volumeSlider" type="range" min="0" max="100" :value="player.volume"
              :style="{ background: volumeBarBg }" @input="handleVolumeChange"
              class="vol-slider w-full h-1.5 rounded-full appearance-none cursor-pointer bg-emphasis outline-none" />
          </div>
        </div>
        <button @click="togglePlayMode" class="text-content-2 hover:text-content transition" :title="modeTitle">
          <svg v-if="player.playMode === 'loop'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 014-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 01-4 4H3"/></svg>
          <svg v-else-if="player.playMode === 'shuffle'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></svg>
          <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 014-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 01-4 4H3"/><text x="11" y="15" font-size="8" fill="currentColor" stroke="none" font-weight="bold">1</text></svg>
        </button>
        <button @click="showQueuePanel = !showQueuePanel"
          class="text-content-2 hover:text-content transition relative" title="播放列表">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
          <span v-if="player.queue.length > 0"
            class="absolute -top-1 -right-1 bg-accent text-content text-xs rounded-full w-4 h-4 flex items-center justify-center">
            {{ player.queue.length }}
          </span>
        </button>
      </div>
    </div>

    <Transition name="slide-up">
      <div v-if="showQueuePanel"
        class="border-t border-line bg-surface/95 backdrop-blur p-4 max-h-64 overflow-y-auto">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-sm font-semibold">播放列表 ({{ player.queue.length }})</h3>
          <button @click="player.clearQueue()" class="text-xs text-danger hover:text-danger transition">清空</button>
        </div>
        <div class="space-y-1">
          <div v-for="(song, idx) in player.queue" :key="song.id + '-' + idx" @click="playFromQueue(idx)" :class="[
            'flex items-center gap-3 p-2 rounded-lg cursor-pointer transition',
            idx === player.currentIndex ? 'bg-accent-dim text-content' : 'hover:bg-subtle text-content-2',
          ]">
            <span class="text-xs w-6 text-center">{{ idx + 1 }}</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-medium truncate">{{ song.name }}</p>
              <p class="text-xs text-content-3 truncate">
                <template v-for="(a, i) in song.ar || []" :key="a.id || i">
                  <span v-if="i > 0" class="text-content-3">/</span>
                  <span class="hover:text-accent-text cursor-pointer transition" @click.stop="a.id && router.push({ name: 'artist', params: { id: a.id } })">{{ a.name }}</span>
                </template>
              </p>
            </div>
            <button @click.stop="player.removeFromQueue(idx)"
              class="text-content-3 hover:text-danger transition text-sm">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount, onMounted } from 'vue';
import { usePlayerStore, PlayMode } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import { formatTime } from '../utils/format';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';

const router = useRouter();
const player = usePlayerStore();
const download = useDownload();
const showQueuePanel = ref(false);
const progressBar = ref<HTMLElement | null>(null);
const isSeeking = ref(false);
const previewTime = ref(0);
const cacheProgress = ref(0);
const prevVolume = ref(100);

let unlistenCache: (() => void) | null = null;

onMounted(async () => {
  const fn = await listen<number>('cache-progress', (event) => {
    cacheProgress.value = event.payload;
  });
  unlistenCache = fn;
});
onBeforeUnmount(() => {
  if (unlistenCache) unlistenCache();
});

const modeTexts = { loop: '列表循环', shuffle: '随机播放', 'repeat-one': '单曲循环' };
const modeTitle = computed(() => modeTexts[player.playMode] || '列表循环');
function togglePlayMode() {
  const modes: PlayMode[] = ['loop', 'shuffle', 'repeat-one'];
  const next = modes[(modes.indexOf(player.playMode) + 1) % modes.length];
  player.setPlayMode(next);
}

function toggleMute() {
  if (player.volume > 0) {
    prevVolume.value = player.volume;
    player.volume = 0;
  } else {
    player.volume = prevVolume.value || 100;
  }
  invoke('set_volume', { vol: player.volume / 100 });
}

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

function playFromQueue(index: number) {
  player.currentIndex = index;
  player.playCurrent();
}

async function handleVolumeChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const val = parseInt(target.value, 10);
  player.volume = val;
  await invoke('set_volume', { vol: val / 100 });
}

const volumeBarBg = computed(() => {
  const pct = player.volume;
  return `linear-gradient(to right, var(--c-accent) 0%, var(--c-accent) ${pct}%, var(--c-muted) ${pct}%)`;
});
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

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
      var(--c-accent) 0%,
      var(--c-accent) var(--vol-fill),
      var(--c-muted) var(--vol-fill));
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
