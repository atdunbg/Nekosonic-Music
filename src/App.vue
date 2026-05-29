<template>
  <div class="flex flex-col h-screen bg-base text-content overflow-hidden">
    <TitleBar @close="closeWindow" />

    <div class="flex flex-1 overflow-hidden" v-if="windowVisible">
      <Sidebar />

      <main class="flex-1 overflow-y-auto pb-24">
        <router-view v-slot="{ Component }">
          <keep-alive :max="5" :include="keepAliveInclude">
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
import { invoke } from '@tauri-apps/api/core';
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
const keepAliveInclude = ref<string[]>(['HomeView', 'DiscoverView', 'FavoriteSongsView', 'DailySongsView', 'LocalMusicView']);

watch(() => settings.dataTheme, (val) => {
  document.documentElement.setAttribute('data-theme', val);
}, { immediate: true });

watch(() => userStore.isLoggedIn, (val) => {
  if (val) {
    player.loadLikedIds();
  }
});

onMounted(async () => {
  document.addEventListener('contextmenu', (e) => e.preventDefault());

  if (userStore.isLoggedIn) {
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

  if (settings.outputDevice) {
    try {
      await invoke('set_output_device', { device: settings.outputDevice });
    } catch { /* 忽略 */ }
  }
});

const currentWindow = getCurrentWindow();

function closeWindow() {
  if (settings.closeAction === 'ask') {
    showCloseModal.value = true;
  } else if (settings.closeAction === 'minimize') {
    currentWindow.hide();
  } else {
    invoke('exit_app');
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
