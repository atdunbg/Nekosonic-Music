<template>
  <div
    class="fixed bottom-0 left-0 right-0 bg-surface/95 backdrop-blur z-50 select-none"
  >
    <div v-if="player.dominantColor"
      class="absolute inset-0 pointer-events-none transition-opacity duration-300"
      :class="drawerActive ? 'opacity-100' : 'opacity-0'"
      :style="{ backgroundColor: player.dominantColor }"
    >
      <div class="absolute inset-0 bg-black/60"></div>
    </div>

    <div ref="progressBar" class="w-full h-1.5 rounded-full relative group cursor-pointer overflow-visible"
      :class="drawerActive ? 'bg-white/10' : 'bg-muted'"
      @mousedown.prevent="startSeek">
      <div class="absolute left-0 top-0 h-full rounded-full" :class="drawerActive ? 'bg-white/20' : 'bg-emphasis'" :style="{ width: cacheProgress + '%' }"></div>
      <div class="absolute left-0 top-0 h-full bg-accent rounded-full"
        :style="{ width: displayProgress + '%' }"></div>
      <div
        class="absolute top-1/2 -translate-y-1/2 w-3.5 h-3.5 rounded-full bg-white shadow-lg border border-line opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
        :style="{ left: `calc(${displayProgress}% - 7px)` }"></div>
    </div>

    <div class="flex items-center px-6 h-16 relative z-10">
      <div class="flex items-center gap-3 w-56 min-w-0">
        <div v-if="getCoverUrl(player.currentSong)" class="w-10 h-10 rounded-md overflow-hidden flex-shrink-0 cursor-pointer hover:scale-105 transition-transform" @click="player.toggleRoamDrawer()" title="全屏展示">
          <img :src="getCoverUrl(player.currentSong)" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-10 h-10 rounded-md flex-shrink-0 flex items-center justify-center cursor-pointer hover:scale-105 transition-transform" @click="player.toggleRoamDrawer()" title="全屏展示"
          :class="drawerActive ? 'bg-white/10' : 'bg-muted'">
          <IconMusic class="w-[18px] h-[18px]" :class="drawerActive ? 'text-white/50' : 'text-content-3'" />
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium truncate" :class="drawerActive ? 'text-white' : ''">{{ player.currentSong?.name }}</p>
          <p class="text-xs truncate" :class="drawerActive ? 'text-white/70' : 'text-content-2'">
            <template v-for="(a, i) in player.currentSong?.ar || []" :key="a.id || i">
              <span v-if="i > 0" :class="drawerActive ? 'text-white/40' : 'text-content-3'">/</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click.stop="a.id && router.push({ name: 'artist', params: { id: a.id } })">{{ a.name }}</span>
            </template>
            <template v-if="player.currentSong?.al?.name">
              <span :class="drawerActive ? 'text-white/40' : 'text-content-3'" class="mx-1">·</span>
              <span class="hover:text-accent-text cursor-pointer transition" @click.stop="player.currentSong!.al.id && router.push({ name: 'album', params: { id: player.currentSong!.al.id } })">{{ player.currentSong.al.name }}</span>
            </template>
          </p>
        </div>
        <button @click="player.currentSong && player.toggleLike(player.currentSong.id)" class="flex-shrink-0 transition" :class="player.currentSong && player.isLiked(player.currentSong.id) ? 'text-danger' : (drawerActive ? 'text-white/50 hover:text-danger' : 'text-content-3 hover:text-danger')">
          <IconHeart v-if="player.currentSong && player.isLiked(player.currentSong.id)" class="w-4 h-4 text-danger [&>path]:fill-current [&>path]:stroke-0" />
          <IconHeart v-else class="w-4 h-4" />
        </button>
        <button v-if="player.currentSong" @click="player.openRoamDrawer('comment')" class="flex-shrink-0 transition" :class="drawerActive ? 'text-white/50 hover:text-white' : 'text-content-3 hover:text-accent-text'" title="评论">
          <IconMessageSquare class="w-4 h-4" />
        </button>
        <button v-if="player.currentSong && !download.isDownloaded(player.currentSong!.id) && !download.isDownloading(player.currentSong!.id)" @click="download.downloadSong(player.currentSong)" class="flex-shrink-0 transition" :class="drawerActive ? 'text-white/50 hover:text-white' : 'text-content-3 hover:text-accent-text'" title="下载">
          <IconDownload class="w-4 h-4" />
        </button>
        <IconLoader2 v-if="player.currentSong && download.isDownloading(player.currentSong!.id)" class="w-4 h-4 flex-shrink-0 animate-spin" :class="drawerActive ? 'text-white/50' : 'text-content-3'" />
      </div>

      <div class="flex-1 flex flex-col items-center justify-center gap-1">
        <div class="flex items-center gap-5 relative">
          <button @click="player.prev()" :disabled="player.isFmMode" :class="[
            'transition',
            player.isFmMode ? (drawerActive ? 'text-white/20 cursor-not-allowed' : 'text-content-4 cursor-not-allowed') : (drawerActive ? 'text-white/70 hover:text-white' : 'text-content-2 hover:text-content'),
          ]">
            <IconSkipBack class="w-5 h-5" />
          </button>
          <button @click="player.toggle()"
            class="w-9 h-9 flex items-center justify-center rounded-full transition"
            :class="drawerActive ? 'bg-white/15 hover:bg-white/25 border border-white/20' : 'bg-muted hover:bg-emphasis border border-emphasis'">
            <IconPause v-if="player.playing" class="w-4 h-4" :class="drawerActive ? 'text-white' : 'text-content'" />
            <IconPlay v-else class="w-4 h-4" :class="drawerActive ? 'text-white' : 'text-content'" />
          </button>
          <button @click="player.next()" :class="drawerActive ? 'text-white/70 hover:text-white transition' : 'text-content-2 hover:text-content transition'">
            <IconSkipForward class="w-5 h-5" />
          </button>
          <button v-if="player.isFmMode && player.currentSong" @click="showDislikeModal = true" class="absolute left-full ml-5" :class="drawerActive ? 'text-white/50 hover:text-danger transition' : 'text-content-3 hover:text-danger transition'" title="减少推荐">
            <IconHeartOff class="w-[18px] h-[18px]" />
          </button>
        </div>
        <div class="flex items-center gap-2 text-xs" :class="drawerActive ? 'text-white/70' : 'text-content-2'">
          <span>{{ formatTime(player.currentTime) }}</span>
          <span>/</span>
          <span>{{ formatTime(player.duration) }}</span>
        </div>
      </div>

      <div class="w-56 flex justify-end items-center gap-2">
        <div class="flex items-center gap-1">
          <button @click="toggleMute" :class="drawerActive ? 'text-white/70 hover:text-white transition' : 'text-content-2 hover:text-content transition'">
            <IconVolumeX v-if="player.volume === 0" class="w-[18px] h-[18px]" />
            <IconVolume2 v-else class="w-[18px] h-[18px]" />
          </button>
          <div class="relative w-20 h-6 flex items-center">
            <input ref="volumeSlider" type="range" min="0" max="100" :value="player.volume"
              :style="{ background: volumeBarBg }" @input="handleVolumeChange"
              class="vol-slider w-full h-1.5 rounded-full appearance-none cursor-pointer outline-none"
              :class="drawerActive ? 'bg-white/20' : 'bg-emphasis'" />
          </div>
        </div>
        <button @click="togglePlayMode" :class="drawerActive ? 'text-white/70 hover:text-white transition' : 'text-content-2 hover:text-content transition'" :title="modeTitle">
          <IconRepeat v-if="player.playMode === 'loop'" class="w-[18px] h-[18px]" />
          <IconShuffle v-else-if="player.playMode === 'shuffle'" class="w-[18px] h-[18px]" />
          <IconRepeat1 v-else class="w-[18px] h-[18px]" />
        </button>
        <button @click="showQueuePanel = !showQueuePanel"
          :class="drawerActive ? 'text-white/70 hover:text-white transition' : 'text-content-2 hover:text-content transition'" title="播放列表">
          <IconListMusic class="w-[18px] h-[18px]" />
        </button>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="queue-fade">
        <div v-if="showDislikeModal && player.currentSong" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showDislikeModal = false">
          <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-5 select-auto">
            <h2 class="text-base font-semibold text-content mb-1">减少推荐</h2>
            <p class="text-xs text-content-3 mb-4">选择要减少的推荐类型</p>
            <div class="flex flex-col gap-2 mb-4">
              <button @click="dislikeSong"
                class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-muted hover:bg-emphasis transition text-left">
                <IconMusic class="w-[18px] h-[18px] text-content-2 flex-shrink-0" />
                <div>
                  <p class="text-sm font-medium">不推荐这首歌曲</p>
                  <p class="text-xs text-content-3 truncate max-w-[200px]">{{ player.currentSong.name }}</p>
                </div>
              </button>
              <button v-if="dislikeArtistName" @click="dislikeArtist"
                class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-muted hover:bg-emphasis transition text-left">
                <IconUserRound class="w-[18px] h-[18px] text-content-2 flex-shrink-0" />
                <div>
                  <p class="text-sm font-medium">不推荐这个歌手</p>
                  <p class="text-xs text-content-3 truncate max-w-[200px]">{{ dislikeArtistName }}</p>
                </div>
              </button>
            </div>
            <button @click="showDislikeModal = false"
              class="w-full py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">
              取消
            </button>
          </div>
        </div>
      </Transition>
      <Transition name="queue-fade">
        <div v-if="showQueuePanel" class="fixed inset-0 z-[55] bg-black/40 backdrop-blur-[2px]" @click="showQueuePanel = false"></div>
      </Transition>
      <Transition name="queue-slide">
        <div v-if="showQueuePanel"
          class="fixed right-0 top-0 bottom-0 z-[56] w-[340px] bg-base/95 backdrop-blur border-l border-line flex flex-col shadow-2xl shadow-black/40">

          <div class="px-5 pt-5 pb-3">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-[1rem] font-semibold text-content">播放列表</h3>
                <p class="text-xs text-content-3 mt-0.5">{{ player.queue.length }} 首歌曲</p>
              </div>
              <div class="flex items-center gap-1">
                <button @click="player.clearQueue()"
                  class="px-2.5 py-1 text-xs text-content-3 hover:text-danger hover:bg-danger-dim rounded-lg transition">
                  清空
                </button>
                <button @click="showQueuePanel = false"
                  class="w-7 h-7 flex items-center justify-center rounded-lg text-content-3 hover:text-content hover:bg-subtle transition">
                  <IconX class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>

          <div class="h-px mx-5 bg-line"></div>

          <div ref="queueListEl" class="flex-1 overflow-y-auto px-3 py-2 relative">
            <div v-if="player.queue.length === 0" class="flex flex-col items-center justify-center h-full text-content-4 gap-3">
              <IconMusic class="w-10 h-10 opacity-40" />
              <p class="text-sm">播放列表为空</p>
              <p class="text-xs text-content-4">去发现好听的音乐吧</p>
            </div>

            <SongListItem
              v-for="(song, idx) in player.queue" :key="song.id + '-' + idx"
              :id="'queue-item-' + idx"
              :song="song"
              :index="idx"
              :is-current="idx === player.currentIndex"
              show-playing-overlay
              cover-size="w-9 h-9"
              cover-size-param="?param=80y80"
              :container-class="idx === player.currentIndex ? 'bg-muted hover:bg-muted gap-3 px-3 py-2 rounded-lg' : 'hover:bg-subtle gap-3 px-3 py-2 rounded-lg'"
              @click="playFromQueue(idx)"
            >
              <template #actions>
                <button @click.stop="player.removeFromQueue(idx)"
                  class="w-6 h-6 flex items-center justify-center rounded-md text-content-4 hover:text-danger hover:bg-danger-dim transition opacity-0 group-hover:opacity-100 flex-shrink-0">
                  <IconX class="w-3 h-3" />
                </button>
              </template>
            </SongListItem>

            <div class="h-2"></div>

            <button v-if="player.currentIndex >= 0 && player.queue.length > 0" v-show="!currentSongVisible"
              @click="scrollToCurrent"
              class="sticky bottom-3 float-right mr-1 w-9 h-9 flex items-center justify-center rounded-full bg-surface/90 backdrop-blur shadow-lg shadow-black/30 text-content-3 hover:text-accent-text hover:bg-accent-dim/50 transition-all duration-300"
              title="定位到正在播放">
              <IconCrosshair class="w-[18px] h-[18px]" />
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount, onMounted, nextTick } from 'vue';
import { usePlayerStore, PlayMode } from '../stores/player';
import { useDownload } from '../composables/useDownload';
import { formatTime } from '../utils/format';
import { getCoverUrl } from '../utils/song';
import { AudioApi } from '../api';
import { showToast } from '../composables/useToast';
import { listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import SongListItem from './SongListItem.vue';
import IconSkipBack from '~icons/lucide/skip-back';
import IconPlay from '~icons/lucide/play';
import IconPause from '~icons/lucide/pause';
import IconSkipForward from '~icons/lucide/skip-forward';
import IconHeartOff from '~icons/lucide/heart-off';
import IconVolumeX from '~icons/lucide/volume-x';
import IconVolume2 from '~icons/lucide/volume-2';
import IconRepeat from '~icons/lucide/repeat';
import IconShuffle from '~icons/lucide/shuffle';
import IconRepeat1 from '~icons/lucide/repeat-1';
import IconListMusic from '~icons/lucide/list-music';
import IconMessageSquare from '~icons/lucide/message-square';
import IconDownload from '~icons/lucide/download';
import IconLoader2 from '~icons/lucide/loader-2';
import IconHeart from '~icons/lucide/heart';
import IconX from '~icons/lucide/x';
import IconMusic from '~icons/lucide/music';
import IconCrosshair from '~icons/lucide/crosshair';
import IconUserRound from '~icons/lucide/user-round';

const router = useRouter();
const player = usePlayerStore();
const download = useDownload();
const drawerActive = computed(() => player.showRoamDrawer && !!player.dominantColor);
const showQueuePanel = ref(false);
const showDislikeModal = ref(false);
const queueListEl = ref<HTMLElement | null>(null);
const currentSongVisible = ref(true);
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

watch(() => player.currentSong, (song) => {
  if (!song) {
    cacheProgress.value = 0;
  } else if (song.localPath) {
    cacheProgress.value = 100;
  } else {
    cacheProgress.value = 0;
  }
});

const modeTexts: Record<PlayMode, string> = { loop: '列表循环', shuffle: '随机播放', 'repeat-one': '单曲循环' };
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
  AudioApi.setVolume(player.volume / 100);
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

const dislikeArtistName = computed(() => {
  const song = player.currentSong;
  if (!song?.ar?.length) return '';
  return song.ar.map(a => a.name).join(' / ');
});

async function dislikeSong() {
  if (!player.currentSong) return;
  showDislikeModal.value = false;
  await player.fmTrash(player.currentSong.id);
  showToast('已减少该歌曲推荐', 'success');
}

async function dislikeArtist() {
  if (!player.currentSong) return;
  showDislikeModal.value = false;
  await player.fmTrash(player.currentSong.id);
  showToast('已减少该歌手推荐', 'success');
}

function scrollToCurrent() {
  const el = document.getElementById('queue-item-' + player.currentIndex);
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

let currentSongObserver: IntersectionObserver | null = null;

function setupCurrentSongObserver() {
  if (currentSongObserver) {
    currentSongObserver.disconnect();
    currentSongObserver = null;
  }
  nextTick(() => {
    const el = document.getElementById('queue-item-' + player.currentIndex);
    if (!el || !queueListEl.value) return;
    currentSongObserver = new IntersectionObserver(
      ([entry]) => { currentSongVisible.value = entry.isIntersecting; },
      { root: queueListEl.value, threshold: 0.5 }
    );
    currentSongObserver.observe(el);
  });
}

watch(() => [showQueuePanel.value, player.currentIndex, player.queue.length], () => {
  if (showQueuePanel.value) {
    setupCurrentSongObserver();
  } else {
    if (currentSongObserver) {
      currentSongObserver.disconnect();
      currentSongObserver = null;
    }
    currentSongVisible.value = true;
  }
});

onBeforeUnmount(() => {
  if (currentSongObserver) {
    currentSongObserver.disconnect();
    currentSongObserver = null;
  }
});

async function handleVolumeChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const val = parseInt(target.value, 10);
  player.volume = val;
  await AudioApi.setVolume(val / 100);
}

const volumeBarBg = computed(() => {
  const pct = player.volume;
  const track = drawerActive.value ? 'rgba(255,255,255,0.2)' : 'var(--c-muted)';
  return `linear-gradient(to right, var(--c-accent) 0%, var(--c-accent) ${pct}%, ${track} ${pct}%)`;
});
</script>

<style scoped>
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

.queue-fade-enter-active,
.queue-fade-leave-active {
  transition: opacity 0.2s ease;
}
.queue-fade-enter-from,
.queue-fade-leave-to {
  opacity: 0;
}

.queue-slide-enter-active,
.queue-slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.queue-slide-enter-from,
.queue-slide-leave-to {
  transform: translateX(100%);
}

</style>
