<template>
  <div class="flex flex-col h-screen bg-base text-content overflow-hidden">
    <div
      data-tauri-drag-region
      class="h-10 flex items-center justify-between px-4 bg-surface/90 backdrop-blur select-none flex-shrink-0"
    >
      <span class="text-xs text-content-3 font-medium ml-2">Nekosonic Music</span>
      <div class="flex items-center gap-1.5">
        <button @click="minimizeWindow" class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition" title="最小化"></button>
        <button @click="toggleMaximize" class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition" title="最大化/还原"></button>
        <button @click="closeWindow" class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition" title="关闭"></button>
      </div>
    </div>

    <div class="flex flex-1 overflow-hidden" v-if="windowVisible">
      <nav class="w-56 flex-shrink-0 flex flex-col bg-surface/80 backdrop-blur">
        <div class="flex-1 p-4 overflow-y-auto min-h-0">
          <div class="flex flex-col min-h-full">
          <div class="space-y-0.5">
            <router-link to="/"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
              active-class="!text-content !bg-muted">
              <IconHome class="w-[18px] h-[18px]" />
              推荐
            </router-link>
            <router-link to="/discover"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
              active-class="!text-content !bg-muted">
              <IconSearch class="w-[18px] h-[18px]" />
              发现
            </router-link>
            <button
              @click="openRoamFromSidebar"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle w-full text-left"
            >
              <IconRadio class="w-[18px] h-[18px]" />
              漫游
            </button>
          </div>

          <div class="mt-4 mb-1 pt-2">
            <p class="text-xs text-content-3 px-3 mb-1">我的</p>
            <div class="space-y-0.5">
              <router-link to="/favorites"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <IconHeart class="w-[18px] h-[18px]" />
                我喜欢的音乐
              </router-link>
              <router-link to="/recent"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <IconClock class="w-[18px] h-[18px]" />
                最近播放
              </router-link>
              <router-link to="/local-music"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <IconMusic class="w-[18px] h-[18px]" />
                本地音乐
              </router-link>
            </div>
          </div>

          <div class="mt-4 mb-1 pt-2" v-if="userStore.isLoggedIn">
            <div class="flex items-center justify-between px-3 mb-1 cursor-pointer group"
              @click="showCreatedPlaylists = !showCreatedPlaylists">
              <p class="text-xs text-content-3">我的歌单</p>
              <IconChevronRight class="w-3 h-3 text-content-3 transition-transform" :class="{ 'rotate-90': showCreatedPlaylists }" />
            </div>
            <div v-show="showCreatedPlaylists" class="space-y-0.5">
              <div v-for="pl in createdPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
                class="flex items-center gap-2.5 px-2 py-1.5 rounded-lg cursor-pointer transition-all duration-200"
                :class="isPlaylistActive(pl.id) ? 'bg-muted' : 'hover:bg-subtle'">
                <img :src="pl.coverImgUrl + '?param=80y80'" class="w-8 h-8 rounded object-cover flex-shrink-0" />
                <span class="text-sm truncate"
                  :class="isPlaylistActive(pl.id) ? 'text-content font-medium' : 'text-content-2'">{{ pl.name }}</span>
              </div>
            </div>
          </div>

          <div class="mt-4 mb-1 pt-2" v-if="userStore.isLoggedIn">
            <div class="flex items-center justify-between px-3 mb-1 cursor-pointer group"
              @click="showSubPlaylists = !showSubPlaylists">
              <p class="text-xs text-content-3">收藏的歌单</p>
              <IconChevronRight class="w-3 h-3 text-content-3 transition-transform" :class="{ 'rotate-90': showSubPlaylists }" />
            </div>
            <div v-show="showSubPlaylists" class="space-y-0.5">
              <div v-for="pl in subPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
                class="flex items-center gap-2.5 px-2 py-1.5 rounded-lg cursor-pointer transition-all duration-200"
                :class="isPlaylistActive(pl.id) ? 'bg-muted' : 'hover:bg-subtle'">
                <img :src="pl.coverImgUrl + '?param=80y80'" class="w-8 h-8 rounded object-cover flex-shrink-0" />
                <span class="text-sm truncate"
                  :class="isPlaylistActive(pl.id) ? 'text-content font-medium' : 'text-content-2'">{{ pl.name }}</span>
              </div>
            </div>
          </div>

          <div class="mt-auto pt-4" :class="player.currentSong ? 'pb-20' : 'pb-2'">
            <div class="px-1">
              <router-link to="/settings"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <IconSettings class="w-[18px] h-[18px]" />
                设置
              </router-link>
            </div>
            <div v-if="!userStore.isLoggedIn" class="mt-3 p-3 rounded-xl bg-subtle/60">
              <p class="text-xs text-content-3 mb-2">强烈建议登录以提升体验</p>
              <router-link to="/login"
                class="flex items-center justify-center gap-2 w-full px-4 py-2 rounded-lg bg-accent hover:bg-accent-hover transition text-sm font-medium text-white">
                <IconLogIn class="w-4 h-4" />
                立即登录
              </router-link>
            </div>
            <div v-else class="flex items-center gap-3 px-2 mt-3">
              <img :src="userStore.user?.avatarUrl" class="w-8 h-8 rounded-full ring-2 ring-accent/50" />
              <div class="min-w-0">
                <p class="text-sm font-medium truncate">{{ userStore.user?.nickname }}</p>
                <button @click="userStore.logout(); player.stop()"
                  class="text-xs text-content-3 hover:text-danger transition">退出登录</button>
              </div>
            </div>
          </div>
          </div>
        </div>
      </nav>

      <main class="flex-1 overflow-y-auto pb-24">
        <router-view v-slot="{ Component }">
          <keep-alive :max="5" :include="keepAliveInclude">
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </main>
    </div>

    <Transition name="drawer">
      <div
        v-if="windowVisible && player.showRoamDrawer"
        class="fixed inset-0 z-50 flex flex-col backdrop-blur-xl"
        :class="!player.dominantColor && 'bg-surface/95'"
        :style="player.dominantColor ? { backgroundColor: player.dominantColor } : {}"
      >
        <div v-if="player.dominantColor" class="absolute inset-0 bg-black/60 pointer-events-none"></div>
        <div class="h-10 flex items-center justify-between px-4 flex-shrink-0 relative z-10" data-tauri-drag-region>
          <button @click="player.closeRoamDrawer()" :class="player.dominantColor ? 'text-white/60 hover:text-white' : 'text-content-2 hover:text-content'" class="transition">
            <IconChevronDown class="w-5 h-5" />
          </button>
          <div class="flex items-center gap-1.5">
            <button @click="minimizeWindow" class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition" title="最小化"></button>
            <button @click="toggleMaximize" class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition" title="最大化/还原"></button>
            <button @click="closeWindow" class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition" title="关闭"></button>
          </div>
        </div>
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
              <button @click="roamTab = 'lyric'"
                class="px-3 py-1 rounded-full text-sm transition"
                :class="player.dominantColor
                  ? (roamTab === 'lyric' ? 'bg-white/15 text-white font-medium' : 'text-white/50 hover:text-white/80')
                  : (roamTab === 'lyric' ? 'bg-muted text-content font-medium' : 'text-content-3 hover:text-content')">
                歌词
              </button>
              <button @click="roamTab = 'comment'"
                class="px-3 py-1 rounded-full text-sm transition"
                :class="player.dominantColor
                  ? (roamTab === 'comment' ? 'bg-white/15 text-white font-medium' : 'text-white/50 hover:text-white/80')
                  : (roamTab === 'comment' ? 'bg-muted text-content font-medium' : 'text-content-3 hover:text-content')">
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
            <div v-show="roamTab === 'lyric'" ref="lyricScrollContainer" class="flex-1 min-h-0 overflow-y-auto custom-scroll px-4">
              <div v-if="lyrics.length > 0" class="w-full max-w-lg mx-auto text-center"
                :style="{ paddingTop: roamLyricPadPx + 'px', paddingBottom: roamLyricPadPx + 'px' }">
                <p
                  v-for="(line, idx) in lyrics"
                  :key="idx"
                  :class="getRoamLyricClass(idx)"
                  class="roam-lyric-line px-4 py-3 rounded-lg cursor-pointer transition-all duration-300"
                  @click="seekToRoamLyric(line.time)"
                  @mouseenter="roamLyricHovering = true"
                  @mouseleave="roamLyricHovering = false"
                >
                  {{ line.text }}
                  <span v-if="showTranslation && line.translation" class="block text-sm opacity-60 mt-1">{{ line.translation }}</span>
                </p>
              </div>
              <div v-else :class="player.dominantColor ? 'text-white/40' : 'text-content-3'" class="text-center mt-8">暂无歌词</div>
            </div>
            <div v-show="roamTab === 'comment'" class="flex-1 min-h-0 overflow-y-auto px-4 pb-4">
              <CommentSection v-if="roamSong" :type="0" :id="player.commentSongId || roamSong.id" :key="player.commentSongId || roamSong.id" />
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <PlayerBar v-if="player.currentSong" />
    <ToastContainer />

    <UpdateDialog
      :visible="updater.updateAvailable.value && !!updater.updateInfo.value"
      :info="{ version: updater.updateInfo.value?.version || '', date: updater.updateInfo.value?.date ?? null, body: updater.updateInfo.value?.body ?? null, currentVersion: updater.currentVersion.value }"
      :downloading="updater.downloading.value"
      :download-progress="updater.downloadProgress.value"
      @update="updater.downloadAndInstall()"
      @ignore="updater.ignoreVersion(updater.updateInfo.value?.version || '')"
    />

    <Transition name="fade">
      <div v-if="showCloseModal" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showCloseModal = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-6 select-auto">
          <h2 class="text-lg font-semibold text-content mb-1">关闭确认</h2>
          <p class="text-sm text-content-2 mb-5">你希望如何处理？</p>
          <div class="space-y-2.5 mb-4">
            <button @click="handleCloseAction('minimize')"
              class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-subtle hover:bg-muted transition text-left">
              <div class="w-9 h-9 rounded-lg bg-accent-dim flex items-center justify-center flex-shrink-0">
                <IconMaximize2 class="w-[18px] h-[18px] text-accent-text" />
              </div>
              <div>
                <p class="text-sm font-medium text-content">最小化到托盘</p>
                <p class="text-xs text-content-3">程序继续在后台运行</p>
              </div>
            </button>
            <button @click="handleCloseAction('exit')"
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
            <input type="checkbox" v-model="closeDontAskAgain" />
            <span class="text-xs text-content-2">不再询问，记住我的选择</span>
          </label>
          <button @click="showCloseModal = false"
            class="w-full py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">
            取消
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useUserStore } from './stores/user';
import { useSettingsStore, type CloseAction } from './stores/settings';
import PlayerBar from './components/PlayerBar.vue';
import ToastContainer from './components/ToastContainer.vue';
import CommentSection from './components/CommentSection.vue';
import UpdateDialog from './components/UpdateDialog.vue';
import { usePlayerStore } from './stores/player';
import { getCoverUrl, extractDominantColor } from './utils/song';
import { useOnlineStatus } from './composables/useOnlineStatus';
import { showToast } from './composables/useToast';
import { useLyric } from './composables/UserLyric';
import { useUpdater } from './composables/useUpdater';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import IconHome from '~icons/lucide/home';
import IconSearch from '~icons/lucide/search';
import IconRadio from '~icons/lucide/radio';
import IconHeart from '~icons/lucide/heart';
import IconSettings from '~icons/lucide/settings';
import IconLogIn from '~icons/lucide/log-in';
import IconChevronDown from '~icons/lucide/chevron-down';
import IconChevronRight from '~icons/lucide/chevron-right';
import IconMaximize2 from '~icons/lucide/maximize-2';
import IconX from '~icons/lucide/x';
import IconClock from '~icons/lucide/clock';
import IconMusic from '~icons/lucide/music';
import IconLanguages from '~icons/lucide/languages';

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();
const player = usePlayerStore();
const settings = useSettingsStore();
const updater = useUpdater();
const { isOnline } = useOnlineStatus();

watch(isOnline, (val, old) => {
  if (val && !old) showToast('网络已恢复', 'success');
  else if (!val && old) showToast('网络已断开，部分功能不可用', 'error');
});

const createdPlaylists = ref<any[]>([]);
const subPlaylists = ref<any[]>([]);
const showCreatedPlaylists = ref(true);
const showSubPlaylists = ref(true);
const showCloseModal = ref(false);
const closeDontAskAgain = ref(false);
const windowVisible = ref(true);
const keepAliveInclude = ref<string[]>(['HomeView', 'DiscoverView', 'FavoriteSongsView', 'DailySongsView', 'LocalMusicView']);

watch(() => settings.dataTheme, (val) => {
  document.documentElement.setAttribute('data-theme', val);
}, { immediate: true });

const { lyrics, currentLyricIdx, hasTranslation, showTranslation, toggleTranslation } = useLyric();
const lyricScrollContainer = ref<HTMLElement | null>(null);
const roamLyricHovering = ref(false);
const roamLyricPadPx = ref(0);
const roamSong = computed(() => player.currentSong);
const roamCoverError = ref(false);
const roamTab = ref<'lyric' | 'comment'>('lyric');
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
    roamTab.value = player.roamInitialTab;
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
  const active = lyricScrollContainer.value.querySelector('.roam-lyric-active') as HTMLElement | null;
  if (active) {
    active.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

function getRoamLyricClass(idx: number): string {
  const diff = Math.abs(idx - currentLyricIdx.value);
  const hasColor = !!player.dominantColor;
  if (idx === currentLyricIdx.value) {
    return 'roam-lyric-active text-accent-text font-semibold text-xl';
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

async function openRoamFromSidebar() {
  if (!userStore.isLoggedIn) {
    router.push('/login');
    return;
  }
  if (player.isFmMode) {
    player.openRoamDrawer();
  } else {
    await player.loadFm();
  }
}

async function loadPlaylists() {
  if (!userStore.isLoggedIn || !userStore.user) return;
  try {
    const jsonStr: string = await invoke('user_playlist', { uid: userStore.user.userId });
    const data = JSON.parse(jsonStr);
    createdPlaylists.value = (data.playlist || []).filter((p: any) => !p.subscribed).slice(1);
    subPlaylists.value = (data.playlist || []).filter((p: any) => p.subscribed);
  } catch { /* 忽略 */ }
}

function goPlaylist(id: number) {
  router.push({ name: 'playlist', params: { id } });
}

function isPlaylistActive(id: number): boolean {
  return route.name === 'playlist' && Number(route.params.id) === id;
}

watch(() => userStore.isLoggedIn, (val) => {
  if (val) {
    loadPlaylists();
    player.loadLikedIds();
  }
});

onMounted(async () => {
  document.addEventListener('contextmenu', (e) => e.preventDefault());

  if (userStore.isLoggedIn) {
    loadPlaylists();
    player.loadLikedIds();
  }
  try { await invoke('stop_audio'); } catch { /* 忽略 */ }
  try {
    const jsonStr: string = await invoke('get_login_status');
    const data = JSON.parse(jsonStr);
    if (data.account || data.profile) {
      const profile = data.profile || data.account;
      userStore.setUser({
        userId: profile.userId,
        nickname: profile.nickname,
        avatarUrl: profile.avatarUrl,
      });
    }
  } catch { /* 忽略 */ }

  updater.checkForUpdate(true);

  // 恢复保存的输出设备设置
  if(settings.outputDevice) {
    try {
      await invoke('set_output_device', { device: settings.outputDevice });
    } catch { /* 忽略 */ }
  }
});

const currentWindow = getCurrentWindow();
function minimizeWindow() { currentWindow.minimize(); }
async function toggleMaximize() {
  const isMaximized = await currentWindow.isMaximized();
  if (isMaximized) { currentWindow.unmaximize(); } else { currentWindow.maximize(); }
}
function closeWindow() {
  if (settings.closeAction === 'ask') {
    closeDontAskAgain.value = false;
    showCloseModal.value = true;
  } else if (settings.closeAction === 'minimize') {
    currentWindow.hide();
  } else {
    invoke('exit_app');
  }
}
function handleCloseAction(action: CloseAction) {
  if (closeDontAskAgain.value) {
    settings.setCloseAction(action);
  }
  showCloseModal.value = false;
  if (action === 'minimize') {
    currentWindow.hide();
  } else {
    invoke('exit_app');
  }
}

onMounted(() => {
  const unlisten1 = listen('tray-play-pause', () => {
    player.toggle();
  });
  const unlisten2 = listen('tray-next', () => {
    player.next();
  });
  const unlisten3 = listen('tray-prev', () => {
    player.prev();
  });
  const unlisten4 = listen('window-hidden', () => {
    windowVisible.value = false;
    keepAliveInclude.value = [];
  });
  const unlisten5 = listen('window-shown', () => {
    windowVisible.value = true;
    keepAliveInclude.value = ['HomeView', 'DiscoverView', 'FavoriteSongsView', 'DailySongsView', 'LocalMusicView'];
  });

  onBeforeUnmount(() => {
    unlisten1.then(fn => fn());
    unlisten2.then(fn => fn());
    unlisten3.then(fn => fn());
    unlisten4.then(fn => fn());
    unlisten5.then(fn => fn());
  });
});

async function registerGlobalShortcuts() {
  const globalActions: Record<string, () => void> = {
    globalPlayPause: () => player.toggle(),
    globalPrev: () => player.prev(),
    globalNext: () => player.next(),
    globalVolUp: () => player.adjustVolume(5),
    globalVolDown: () => player.adjustVolume(-5),
  };
  for (const [id, action] of Object.entries(globalActions)) {
    const key = settings.shortcuts[id]?.key;
    if (!key) continue;
    try { await unregister(key); } catch { /* 忽略 */ }
    try {
      await register(key, (event) => {
        if (event.state === 'Pressed') action();
      });
    } catch { /* 忽略 */ }
  }
}

watch(() => settings.shortcuts, () => {
  registerGlobalShortcuts();
}, { deep: true });

onMounted(() => {
  registerGlobalShortcuts();
});

function parseShortcutKey(combo: string): { ctrl: boolean; alt: boolean; shift: boolean; code: string } {
  const parts = combo.split('+');
  return {
    ctrl: parts.includes('Control'),
    alt: parts.includes('Alt'),
    shift: parts.includes('Shift'),
    code: parts.find(p => !['Control', 'Alt', 'Shift'].includes(p)) || '',
  };
}

onMounted(() => {
  function onKeydown(e: KeyboardEvent) {
    const el = e.target as HTMLElement;
    const isEditable = el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable;
    if (e.code === 'Space' && !isEditable) {
      e.preventDefault();
      player.toggle();
    }

    const localActions: Record<string, () => void> = {
      playPause: () => player.toggle(),
      prev: () => player.prev(),
      next: () => player.next(),
      volUp: () => player.adjustVolume(5),
      volDown: () => player.adjustVolume(-5),
    };
    for (const [id, action] of Object.entries(localActions)) {
      const key = settings.shortcuts[id]?.key;
      if (!key) continue;
      const parsed = parseShortcutKey(key);
      const ctrlMatch = parsed.ctrl ? (e.ctrlKey || e.metaKey) : !e.ctrlKey && !e.metaKey;
      const altMatch = parsed.alt ? e.altKey : !e.altKey;
      const shiftMatch = parsed.shift ? e.shiftKey : !e.shiftKey;
      if (ctrlMatch && altMatch && shiftMatch && e.code === parsed.code) {
        e.preventDefault();
        action();
        return;
      }
    }
  }
  window.addEventListener('keydown', onKeydown);
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeydown);
  });
});
</script>

<style>
.drawer-enter-active,
.drawer-leave-active { transition: transform 0.3s ease; }
.drawer-enter-from,
.drawer-leave-to { transform: translateY(100%); }
.fade-enter-active,
.fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from,
.fade-leave-to { opacity: 0; }
.custom-scroll::-webkit-scrollbar { width: 0; display: none; }
.roam-lyric-line:hover {
  background: var(--c-subtle);
}
.roam-lyric-active:hover {
  background: var(--c-subtle) !important;
}
</style>
