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

    <div class="flex flex-1 overflow-hidden">
      <nav class="w-56 flex-shrink-0 flex flex-col bg-surface/80 backdrop-blur">
        <div class="flex-1 p-4 overflow-y-auto min-h-0">
          <div class="flex flex-col min-h-full">
          <div class="relative mb-4">
            <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-content-3" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg>
            <input v-model="searchQuery" @keydown.enter="doSearch" type="text" placeholder="搜索音乐..."
              class="w-full rounded-lg bg-subtle pl-9 pr-3 py-2 text-sm text-content placeholder-content-3 outline-none focus:bg-muted transition" />
          </div>
          <div class="space-y-0.5">
            <router-link to="/"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
              active-class="!text-content !bg-muted">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12l9-9 9 9"/><path d="M5 10v10a1 1 0 001 1h3v-6h6v6h3a1 1 0 001-1V10"/></svg>
              推荐
            </router-link>
            <button
              @click="openRoamFromSidebar"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle w-full text-left"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9"/><path d="M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.4"/><path d="M16.2 7.8c2.3 2.3 2.3 6.1 0 8.4"/><path d="M19.1 4.9C23 8.8 23 15.1 19.1 19"/></svg>
              漫游
            </button>
          </div>

          <div class="mt-4 mb-1 pt-2">
            <p class="text-xs text-content-3 px-3 mb-1">我的</p>
            <div class="space-y-0.5">
              <router-link to="/favorites"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
                我喜欢的音乐
              </router-link>
              <router-link to="/recent"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
                最近播放
              </router-link>
            </div>
          </div>

          <div class="mt-4 mb-1 pt-2" v-if="userStore.isLoggedIn">
            <div class="flex items-center justify-between px-3 mb-1 cursor-pointer"
              @click="showCreatedPlaylists = !showCreatedPlaylists">
              <p class="text-xs text-content-3">我的歌单</p>
              <span class="text-xs text-content-3 transition-transform"
                :class="{ 'rotate-90': showCreatedPlaylists }">▶</span>
            </div>
            <div v-show="showCreatedPlaylists" class="space-y-0.5">
              <div v-for="pl in createdPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
                class="px-3 py-1.5 rounded-lg text-sm cursor-pointer truncate transition-all duration-200"
                :class="isPlaylistActive(pl.id) ? 'text-content bg-muted' : 'text-content-2 hover:text-content hover:bg-subtle'">
                {{ pl.name }}
              </div>
            </div>
          </div>

          <div class="mt-4 mb-1 pt-2" v-if="userStore.isLoggedIn">
            <div class="flex items-center justify-between px-3 mb-1 cursor-pointer"
              @click="showSubPlaylists = !showSubPlaylists">
              <p class="text-xs text-content-3">收藏的歌单</p>
              <span class="text-xs text-content-3 transition-transform" :class="{ 'rotate-90': showSubPlaylists }">▶</span>
            </div>
            <div v-show="showSubPlaylists" class="space-y-0.5">
              <div v-for="pl in subPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
                class="px-3 py-1.5 rounded-lg text-sm cursor-pointer truncate transition-all duration-200"
                :class="isPlaylistActive(pl.id) ? 'text-content bg-muted' : 'text-content-2 hover:text-content hover:bg-subtle'">
                {{ pl.name }}
              </div>
            </div>
          </div>

          <div class="mt-auto pt-4" :class="player.currentSong ? 'pb-20' : 'pb-2'">
            <div class="px-1">
              <router-link to="/settings"
                class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
                active-class="!text-content !bg-muted">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
                设置
              </router-link>
            </div>
            <div v-if="!userStore.isLoggedIn" class="mt-3 p-3 rounded-xl bg-subtle/60">
              <p class="text-xs text-content-3 mb-2">强烈建议登录以提升体验</p>
              <router-link to="/login"
                class="flex items-center justify-center gap-2 w-full px-4 py-2 rounded-lg bg-accent hover:bg-accent-hover transition text-sm font-medium text-white">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h4a2 2 0 012 2v14a2 2 0 01-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
                立即登录
              </router-link>
            </div>
            <div v-else class="flex items-center gap-3 px-2 mt-3">
              <img :src="userStore.user?.avatarUrl" class="w-8 h-8 rounded-full ring-2 ring-accent/50" />
              <div class="min-w-0">
                <p class="text-sm font-medium truncate">{{ userStore.user?.nickname }}</p>
                <button @click="userStore.logout()"
                  class="text-xs text-content-3 hover:text-danger transition">退出登录</button>
              </div>
            </div>
          </div>
          </div>
        </div>
      </nav>

      <main class="flex-1 overflow-y-auto pb-24">
        <router-view v-slot="{ Component }">
          <keep-alive :max="3" include="HomeView,DiscoverView">
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </main>
    </div>

    <Transition name="drawer">
      <div
        v-if="player.showRoamDrawer"
        class="fixed inset-0 z-50 flex flex-col backdrop-blur-xl bg-black/80"
      >
        <div class="h-10 flex items-center justify-between px-4 flex-shrink-0" data-tauri-drag-region>
          <button @click="player.closeRoamDrawer()" class="text-content-2 hover:text-content transition">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
          </button>
          <div class="flex items-center gap-1.5">
            <button @click="minimizeWindow" class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition" title="最小化"></button>
            <button @click="toggleMaximize" class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition" title="最大化/还原"></button>
            <button @click="closeWindow" class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition" title="关闭"></button>
          </div>
        </div>
        <div class="flex-1 min-h-0 flex px-8 pb-8 gap-0">
          <div class="w-2/5 flex flex-col items-center justify-center flex-shrink-0">
            <img
              :src="roamSong?.al?.picUrl || roamSong?.album?.picUrl"
              class="w-72 h-72 rounded-3xl object-cover shadow-2xl mb-4"
            />
            <h1 class="text-2xl font-bold text-white text-center">{{ roamSong?.name }}</h1>
            <p class="text-content-2 mt-2 text-center">{{ roamArtists }}</p>
          </div>
          <div class="w-3/5 relative min-h-0 overflow-hidden flex flex-col">
            <div ref="lyricScrollContainer" class="h-full overflow-y-auto custom-scroll px-4">
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
                </p>
              </div>
              <div v-else class="text-content-3 text-center mt-8">暂无歌词</div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <PlayerBar v-if="player.currentSong" />
    <ToastContainer />

    <Transition name="fade">
      <div v-if="showCloseModal" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showCloseModal = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-6 select-auto">
          <h2 class="text-lg font-semibold text-content mb-1">关闭确认</h2>
          <p class="text-sm text-content-2 mb-5">你希望如何处理？</p>
          <div class="space-y-2.5 mb-4">
            <button @click="handleCloseAction('minimize')"
              class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-subtle hover:bg-muted transition text-left">
              <div class="w-9 h-9 rounded-lg bg-accent-dim flex items-center justify-center flex-shrink-0">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent-text"><path d="M8 3v3a2 2 0 01-2 2H3m18 0h-3a2 2 0 01-2-2V3m0 18v-3a2 2 0 012-2h3M3 16h3a2 2 0 012 2v3"/></svg>
              </div>
              <div>
                <p class="text-sm font-medium text-content">最小化到托盘</p>
                <p class="text-xs text-content-3">程序继续在后台运行</p>
              </div>
            </button>
            <button @click="handleCloseAction('exit')"
              class="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-subtle hover:bg-muted transition text-left">
              <div class="w-9 h-9 rounded-lg bg-danger-dim flex items-center justify-center flex-shrink-0">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-danger"><path d="M18 6L6 18M6 6l12 12"/></svg>
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
import { usePlayerStore } from './stores/player';
import { useLyric } from './composables/UserLyric';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();
const player = usePlayerStore();
const settings = useSettingsStore();

const createdPlaylists = ref<any[]>([]);
const subPlaylists = ref<any[]>([]);
const showCreatedPlaylists = ref(true);
const showSubPlaylists = ref(true);
const searchQuery = ref('');
const showCloseModal = ref(false);
const closeDontAskAgain = ref(false);

watch(() => settings.theme, (val) => {
  document.documentElement.setAttribute('data-theme', val);
}, { immediate: true });

function doSearch() {
  const q = searchQuery.value.trim();
  if (q) router.push({ path: '/discover', query: { q } });
}

const { lyrics, currentLyricIdx } = useLyric();
const lyricScrollContainer = ref<HTMLElement | null>(null);
const roamLyricHovering = ref(false);
const roamLyricPadPx = ref(0);
const roamSong = computed(() => player.currentSong);
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
const roamArtists = computed(() => {
  if (!roamSong.value) return '';
  return roamSong.value.ar?.map((a: any) => a.name).join(' / ') || '';
});

watch(currentLyricIdx, () => {
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
  if (idx === currentLyricIdx.value) {
    return 'roam-lyric-active text-accent-text font-semibold text-xl';
  }
  if (diff === 1) return 'text-content/70 text-lg';
  if (diff === 2) return 'text-content-2/50 text-base';
  return 'text-content-3/35 text-base';
}

function seekToRoamLyric(time: number) {
  if (time != null && player.duration > 0) {
    player.seek(time);
  }
}

async function openRoamFromSidebar() {
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
  } catch (e) { /* 忽略 */ }
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
  if (userStore.isLoggedIn) {
    loadPlaylists();
    player.loadLikedIds();
  }
  try { await invoke('stop_audio'); } catch {}
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
  } catch {}
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

  onBeforeUnmount(() => {
    unlisten1.then(fn => fn());
    unlisten2.then(fn => fn());
    unlisten3.then(fn => fn());
  });
});

onMounted(() => {
  function onKeydown(e: KeyboardEvent) {
    const el = e.target as HTMLElement;
    const isEditable = el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable;
    if (e.code === 'Space' && !isEditable) {
      e.preventDefault();
      player.toggle();
    }
    if ((e.ctrlKey || e.metaKey) && e.code === 'ArrowRight') {
      e.preventDefault();
      player.next();
    }
    if ((e.ctrlKey || e.metaKey) && e.code === 'ArrowLeft') {
      e.preventDefault();
      player.prev();
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
  color: var(--c-content) !important;
}
.roam-lyric-active:hover {
  background: var(--c-subtle) !important;
  color: var(--c-content) !important;
}
</style>
