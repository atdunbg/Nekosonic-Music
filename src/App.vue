<template>
  <div class="flex flex-col h-screen bg-gray-950 text-white overflow-hidden">
    <!-- ========= 自定义标题栏（可拖拽、无边框） ========= -->
    <div
      data-tauri-drag-region
      class="h-10 flex items-center justify-between px-4 bg-gray-900/90 backdrop-blur select-none flex-shrink-0"
    >
      <span class="text-xs text-gray-400 font-medium ml-2">Nekosonic Music</span>
      <div class="flex items-center gap-1.5">
        <!-- 最小化 -->
        <button @click="minimizeWindow" class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition" title="最小化"></button>
        <!-- 最大化 / 还原 -->
        <button @click="toggleMaximize" class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition" title="最大化/还原"></button>
        <!-- 关闭 -->
        <button @click="closeWindow" class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition" title="关闭"></button>
      </div>
    </div>

    <!-- 主体内容区 -->
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧导航（无边框） -->
      <nav class="w-56 flex-shrink-0 flex flex-col bg-gray-900/80 backdrop-blur">
        <div class="flex-1 p-4 overflow-y-auto pb-24 flex flex-col">
          <!-- 推荐 & 发现 -->
          <div class="space-y-0.5">
            <router-link to="/"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-white/60 hover:text-white hover:bg-white/5"
              active-class="!text-white !bg-white/10">
              <span>🏠</span> 推荐
            </router-link>
            <router-link to="/discover"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-white/60 hover:text-white hover:bg-white/5"
              active-class="!text-white !bg-white/10">
              <span>🔍</span> 发现
            </router-link>
            <button
              @click="openRoamFromSidebar"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-white/60 hover:text-white hover:bg-white/5 w-full text-left"
            >
              <span>🌀</span> 漫游
            </button>
          </div>

          <!-- 我的 -->
          <div class="mt-4 mb-1 pt-2">
            <p class="text-xs text-gray-500 px-3 mb-1">我的</p>
            <router-link to="/favorites"
              class="flex items-center gap-3 px-3 py-1.5 rounded-lg text-sm text-white/60 hover:text-white hover:bg-white/5 transition">
              <span>❤️</span> 我喜欢的音乐
            </router-link>
            <router-link to="/recent"
              class="flex items-center gap-3 px-3 py-1.5 rounded-lg text-sm text-white/60 hover:text-white hover:bg-white/5 transition">
              <span>🕐</span> 最近播放
            </router-link>
          </div>

          <!-- 创建的歌单（可折叠） -->
          <div class="mt-4 mb-1 pt-2" v-if="userStore.isLoggedIn">
            <div class="flex items-center justify-between px-3 mb-1 cursor-pointer"
              @click="showCreatedPlaylists = !showCreatedPlaylists">
              <p class="text-xs text-gray-500">我的歌单</p>
              <span class="text-xs text-gray-500 transition-transform"
                :class="{ 'rotate-90': showCreatedPlaylists }">▶</span>
            </div>
            <div v-show="showCreatedPlaylists" class="space-y-0.5">
              <div v-for="pl in createdPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
                class="px-3 py-1.5 rounded-lg text-sm text-white/60 hover:text-white hover:bg-white/5 cursor-pointer truncate transition">
                {{ pl.name }}
              </div>
            </div>
          </div>

          <!-- 收藏的歌单（可折叠） -->
          <div class="mt-4 mb-1 pt-2" v-if="userStore.isLoggedIn">
            <div class="flex items-center justify-between px-3 mb-1 cursor-pointer"
              @click="showSubPlaylists = !showSubPlaylists">
              <p class="text-xs text-gray-500">收藏的歌单</p>
              <span class="text-xs text-gray-500 transition-transform" :class="{ 'rotate-90': showSubPlaylists }">▶</span>
            </div>
            <div v-show="showSubPlaylists" class="space-y-0.5">
              <div v-for="pl in subPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
                class="px-3 py-1.5 rounded-lg text-sm text-white/60 hover:text-white hover:bg-white/5 cursor-pointer truncate transition">
                {{ pl.name }}
              </div>
            </div>
          </div>

          <!-- 用户区域 -->
          <div class="mt-auto pt-4">
            <div v-if="!userStore.isLoggedIn" class="px-2 space-y-2">
              <p class="text-xs text-gray-500">登录后享受个人歌单</p>
              <router-link to="/login"
                class="flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-white/5 hover:bg-white/10 transition text-sm font-medium text-green-400">
                <span>🔑</span> 立即登录
              </router-link>
            </div>
            <div v-else class="flex items-center gap-3 px-2">
              <img :src="userStore.user?.avatarUrl" class="w-8 h-8 rounded-full ring-2 ring-green-400/50" />
              <div class="min-w-0">
                <p class="text-sm font-medium truncate">{{ userStore.user?.nickname }}</p>
                <button @click="userStore.logout()"
                  class="text-xs text-gray-500 hover:text-red-400 transition">退出登录</button>
              </div>
            </div>
          </div>
        </div>
      </nav>

      <!-- 主内容区 -->
      <main class="flex-1 overflow-y-auto pb-24">
        <router-view v-slot="{ Component }">
          <keep-alive :max="3" include="HomeView,DiscoverView">
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </main>
    </div>

    <!-- 全屏漫游抽屉 -->
    <Transition name="drawer">
      <div
        v-if="player.showRoamDrawer"
        class="fixed inset-0 z-50 flex flex-col backdrop-blur-xl bg-black/80"
      >
        <div class="h-16 flex items-center px-6 flex-shrink-0">
          <button @click="player.closeRoamDrawer()" class="text-white/80 hover:text-white transition">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="flex-1 min-h-0 flex px-8 pb-8">
          <div class="flex-shrink-0 mr-12 flex flex-col items-center self-center">
            <img
              :src="roamSong?.al?.picUrl || roamSong?.album?.picUrl"
              class="w-72 h-72 rounded-3xl object-cover shadow-2xl mb-4"
            />
            <h1 class="text-2xl font-bold text-white">{{ roamSong?.name }}</h1>
            <p class="text-gray-400 mt-2">{{ roamArtists }}</p>
          </div>
          <div ref="lyricScrollContainer" class="flex-1 min-h-0 overflow-y-auto custom-scroll px-4">
            <div v-if="lyrics.length > 0" class="w-full max-w-lg mx-auto text-center space-y-3 py-8">
              <p
                v-for="(line, idx) in lyrics"
                :key="idx"
                :class="idx === currentLyricIdx ? 'text-green-400 font-medium text-lg transition' : 'text-gray-400 text-base'"
              >
                {{ line.text }}
              </p>
            </div>
            <div v-else class="text-gray-500 text-center mt-8">暂无歌词</div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 底部播放栏 -->
    <PlayerBar />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useUserStore } from './stores/user';
import PlayerBar from './components/PlayerBar.vue';
import { usePlayerStore } from './stores/player';
import { useLyric } from './composables/UserLyric';
import { getCurrentWindow } from '@tauri-apps/api/window';

const router = useRouter();
const userStore = useUserStore();
const player = usePlayerStore();

const createdPlaylists = ref<any[]>([]);
const subPlaylists = ref<any[]>([]);
const showCreatedPlaylists = ref(true);
const showSubPlaylists = ref(true);

// 歌词
const { lyrics, currentLyricIdx } = useLyric();
const lyricScrollContainer = ref<HTMLElement | null>(null);
const roamSong = computed(() => player.currentSong);
const roamArtists = computed(() => {
  if (!roamSong.value) return '';
  return roamSong.value.ar?.map((a: any) => a.name).join(' / ') || '';
});

watch(currentLyricIdx, () => {
  if (player.showRoamDrawer && lyricScrollContainer.value) {
    nextTick(() => {
      const active = lyricScrollContainer.value?.querySelector('.text-green-400');
      active?.scrollIntoView({ behavior: 'smooth', block: 'center' });
    });
  }
});

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
    createdPlaylists.value = (data.playlist || []).filter((p: any) => !p.subscribed);
    subPlaylists.value = (data.playlist || []).filter((p: any) => p.subscribed);
  } catch (e) { /* 忽略 */ }
}

function goPlaylist(id: number) {
  router.push({ name: 'playlist', params: { id } });
}

watch(() => userStore.isLoggedIn, (val) => {
  if (val) loadPlaylists();
});

onMounted(async () => {
  if (userStore.isLoggedIn) loadPlaylists();
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

// ---------- 窗口控制 ----------
const currentWindow = getCurrentWindow();
function minimizeWindow() { currentWindow.minimize(); }
async function toggleMaximize() {
  const isMaximized = await currentWindow.isMaximized();
  if (isMaximized) { currentWindow.unmaximize(); } else { currentWindow.maximize(); }
}
function closeWindow() { currentWindow.close(); }


import { listen } from '@tauri-apps/api/event';

onMounted(() => {
  const unlisten1 = listen('tray-play-pause', () => {
    player.toggle(); // 假设 player 是 usePlayerStore 的实例
  });
  const unlisten2 = listen('tray-next', () => {
    player.next();
  });
  const unlisten3 = listen('tray-prev', () => {
    player.prev();
  });

  // 在组件卸载时取消监听
onBeforeUnmount(() => {
    unlisten1.then(fn => fn());
    unlisten2.then(fn => fn());
    unlisten3.then(fn => fn());
  });
});

</script>

<style>
.drawer-enter-active,
.drawer-leave-active { transition: transform 0.3s ease; }
.drawer-enter-from,
.drawer-leave-to { transform: translateY(100%); }
.custom-scroll::-webkit-scrollbar { width: 4px; }
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}
</style>