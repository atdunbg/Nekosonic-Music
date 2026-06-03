<template>
  <div class="flex flex-col h-screen bg-base text-content overflow-hidden">
    <TitleBar @close="closeWindow" />

    <div class="flex flex-1 overflow-hidden" v-if="windowVisible">
      <Sidebar />

      <main class="flex-1 overflow-y-auto pb-24">
        <router-view v-slot="{ Component }">
          <keep-alive :max="10" :include="keepAliveInclude">
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </main>
    </div>

    <RoamDrawer :visible="windowVisible && player.showRoamDrawer" />

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

    <CloseModal
      :visible="showCloseModal"
      @confirm="handleCloseAction"
      @cancel="showCloseModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import { useRoute } from 'vue-router';
import { useUserStore } from './stores/user';
import { useSettingsStore, type CloseAction } from './stores/settings';
import { usePlayerStore } from './stores/player';
import TitleBar from './components/TitleBar.vue';
import Sidebar from './components/Sidebar.vue';
import RoamDrawer from './components/RoamDrawer.vue';
import PlayerBar from './components/PlayerBar.vue';
import ToastContainer from './components/ToastContainer.vue';
import CloseModal from './components/CloseModal.vue';
import UpdateDialog from './components/UpdateDialog.vue';
import { useOnlineStatus } from './composables/useOnlineStatus';
import { showToast } from './composables/useToast';
import { useUpdater } from './composables/useUpdater';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { MusicApi, AudioApi, DeviceApi, AppApi } from './api';

const userStore = useUserStore();
const player = usePlayerStore();
const settings = useSettingsStore();
const updater = useUpdater();
const { isOnline } = useOnlineStatus();

watch(isOnline, (val, old) => {
  if (val && !old) showToast('网络已恢复', 'success');
  else if (!val && old) showToast('网络已断开，部分功能不可用', 'error');
});

const showCloseModal = ref(false);
const windowVisible = ref(true);

// --- Keep-alive 缓存管理 ---
// 规则：30秒未访问的页面自动清除缓存；多级跳转时保留导航链上的页面；FavoriteSongs 常驻
const route = useRoute();

const ROUTE_COMPONENT: Record<string, string> = {
  home: 'HomeView', discover: 'DiscoverView', search: 'DiscoverView',
  favorites: 'FavoriteSongsView', daily: 'DailySongsView',
  'local-music': 'LocalMusicView', 'downloaded-music': 'DownloadedMusicView',
  'cloud-music': 'CloudMusicView',
  playlist: 'PlaylistDetailView', artist: 'ArtistDetailView', album: 'AlbumDetailView',
};
const ALL_CACHEABLE = [...new Set(Object.values(ROUTE_COMPONENT))];
const PERMANENT = new Set(['FavoriteSongsView']);
const CACHE_TTL = 30_000;

const lastActivatedAt: Record<string, number> = {};
const navStack = ref<string[]>([]);
const currentComp = ref('');
for (const name of ALL_CACHEABLE) lastActivatedAt[name] = Date.now();

watch(() => route.name, (newName, oldName) => {
  // 离开旧页面时刷新其计时（30s 从离开时算起）
  const oldComp = ROUTE_COMPONENT[oldName as string];
  if (oldComp) lastActivatedAt[oldComp] = Date.now();

  const comp = ROUTE_COMPONENT[newName as string];
  if (!comp) return;
  currentComp.value = comp;
  lastActivatedAt[comp] = Date.now();
  const idx = navStack.value.indexOf(comp);
  if (idx !== -1) {
    // 返回：截断到该位置
    navStack.value = navStack.value.slice(0, idx + 1);
  } else {
    navStack.value.push(comp);
  }
}, { immediate: true });

function computeInclude(): string[] {
  const now = Date.now();
  const include = new Set<string>(PERMANENT);
  if (currentComp.value) include.add(currentComp.value);
  for (const name of navStack.value) include.add(name);
  for (const name of ALL_CACHEABLE) {
    if (lastActivatedAt[name] && now - lastActivatedAt[name] < CACHE_TTL) include.add(name);
  }
  return [...include];
}

const keepAliveInclude = ref<string[]>(computeInclude());
let cleanupTimer: ReturnType<typeof setInterval>;
function startCleanup() { cleanupTimer = setInterval(() => { keepAliveInclude.value = computeInclude(); }, 10_000); }
function stopCleanup() { clearInterval(cleanupTimer); }

watch(() => settings.dataTheme, (val) => {
  document.documentElement.setAttribute('data-theme', val);
}, { immediate: true });

watch(() => userStore.isLoggedIn, (val) => {
  if (val) {
    player.loadLikedIds();
  }
});

onMounted(() => {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
  startCleanup();

  AudioApi.stopAudio().catch(() => {});

  if (userStore.isLoggedIn) {
    player.loadLikedIds();
    MusicApi.getLoginStatus().then(jsonStr => {
      if (!jsonStr) return;
      const data = JSON.parse(jsonStr);
      if (data.account || data.profile) {
        const profile = data.profile || data.account;
        userStore.setUser({
          userId: profile.userId,
          nickname: profile.nickname,
          avatarUrl: profile.avatarUrl,
        });
      }
    }).catch(() => {});
  }

  updater.checkForUpdate(true);

  if (settings.outputDevice) {
    DeviceApi.setOutputDevice(settings.outputDevice).catch(() => {});
  }
});

const currentWindow = getCurrentWindow();

function closeWindow() {
  if (settings.closeAction === 'ask') {
    showCloseModal.value = true;
  } else if (settings.closeAction === 'minimize') {
    currentWindow.hide();
  } else {
    AppApi.exitApp();
  }
}

function handleCloseAction(action: CloseAction, remember: boolean) {
  if (remember) {
    settings.setCloseAction(action);
  }
  showCloseModal.value = false;
  if (action === 'minimize') {
    currentWindow.hide();
  } else {
    AppApi.exitApp();
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
    stopCleanup();
  });
  const unlisten5 = listen('window-shown', () => {
    windowVisible.value = true;
    keepAliveInclude.value = computeInclude();
    startCleanup();
  });

  onBeforeUnmount(() => {
    unlisten1.then(fn => fn());
    unlisten2.then(fn => fn());
    unlisten3.then(fn => fn());
    unlisten4.then(fn => fn());
    unlisten5.then(fn => fn());
    stopCleanup();
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
