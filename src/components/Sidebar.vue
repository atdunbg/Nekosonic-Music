<template>
  <nav class="w-56 flex-shrink-0 flex flex-col" :style="sidebarBgStyle">
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
            <router-link to="/downloaded-music"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
              active-class="!text-content !bg-muted">
              <IconDownload class="w-[18px] h-[18px]" />
              下载音乐
            </router-link>
            <router-link v-if="userStore.isLoggedIn" to="/cloud-music"
              class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 text-content-2 hover:text-content hover:bg-subtle"
              active-class="!text-content !bg-muted">
              <IconCloud class="w-[18px] h-[18px]" />
              音乐云盘
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
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useUserStore } from '../stores/user';
import { usePlayerStore } from '../stores/player';
import { useSettingsStore } from '../stores/settings';
import { MusicApi } from '../api';
import IconHome from '~icons/lucide/home';
import IconSearch from '~icons/lucide/search';
import IconRadio from '~icons/lucide/radio';
import IconHeart from '~icons/lucide/heart';
import IconSettings from '~icons/lucide/settings';
import IconLogIn from '~icons/lucide/log-in';
import IconChevronRight from '~icons/lucide/chevron-right';
import IconClock from '~icons/lucide/clock';
import IconMusic from '~icons/lucide/music';
import IconCloud from '~icons/lucide/cloud';
import IconDownload from '~icons/lucide/download';

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();
const player = usePlayerStore();
const settings = useSettingsStore();

// 有壁纸时侧栏轻微半透明区分区域，无壁纸时保持原样
const sidebarBgStyle = computed(() => {
  if (settings.currentWallpaper.path) return {}; // 有壁纸时透明，由遮罩层统一提供背景
  return { backgroundColor: settings.currentColors.surface };
});

const createdPlaylists = ref<any[]>([]);
const subPlaylists = ref<any[]>([]);
const showCreatedPlaylists = ref(true);
const showSubPlaylists = ref(true);

async function loadPlaylists() {
  if (!userStore.isLoggedIn || !userStore.user) return;
  try {
    const jsonStr: string = await MusicApi.userPlaylist(userStore.user.userId);
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

watch(() => userStore.isLoggedIn, (val) => {
  if (val) {
    loadPlaylists();
    player.loadLikedIds();
  }
});

onMounted(() => {
  if (userStore.isLoggedIn) {
    loadPlaylists();
  }
});
</script>
