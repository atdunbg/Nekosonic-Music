<template>
  <div class="flex-1 overflow-y-auto min-h-0" :class="collapsed ? 'p-2' : 'p-4'">
    <div class="flex flex-col min-h-full">
      <!-- 主导航 -->
      <div class="space-y-0.5">
        <router-link to="/"
          :class="navItemClass"
          active-class="!text-content !bg-muted"
          :title="collapsed ? '推荐' : undefined">
          <IconHome class="w-[18px] h-[18px] flex-shrink-0" />
          <span class="nav-label" :class="labelClass">推荐</span>
        </router-link>
        <router-link to="/discover"
          :class="navItemClass"
          active-class="!text-content !bg-muted"
          :title="collapsed ? '发现' : undefined">
          <IconSearch class="w-[18px] h-[18px] flex-shrink-0" />
          <span class="nav-label" :class="labelClass">发现</span>
        </router-link>
        <button
          @click="openRoamFromSidebar"
          :class="navItemClass"
          :title="collapsed ? '漫游' : undefined">
          <IconRadio class="w-[18px] h-[18px] flex-shrink-0" />
          <span class="nav-label" :class="labelClass">漫游</span>
        </button>
      </div>

      <!-- 我的 -->
      <div class="mt-4 mb-1 pt-2">
        <p v-if="!collapsed" class="text-xs text-content-3 px-3 mb-1">我的</p>
        <div v-else class="border-t border-line-2/50 my-2"></div>
        <div class="space-y-0.5">
          <router-link to="/favorites"
            :class="navItemClass"
            active-class="!text-content !bg-muted"
            :title="collapsed ? '我喜欢的音乐' : undefined">
            <IconHeart class="w-[18px] h-[18px] flex-shrink-0" />
            <span class="nav-label" :class="labelClass">我喜欢的音乐</span>
          </router-link>
          <router-link to="/recent"
            :class="navItemClass"
            active-class="!text-content !bg-muted"
            :title="collapsed ? '最近播放' : undefined">
            <IconClock class="w-[18px] h-[18px] flex-shrink-0" />
            <span class="nav-label" :class="labelClass">最近播放</span>
          </router-link>
          <router-link to="/local-music"
            :class="navItemClass"
            active-class="!text-content !bg-muted"
            :title="collapsed ? '本地音乐' : undefined">
            <IconMusic class="w-[18px] h-[18px] flex-shrink-0" />
            <span class="nav-label" :class="labelClass">本地音乐</span>
          </router-link>
          <router-link to="/downloaded-music"
            :class="navItemClass"
            active-class="!text-content !bg-muted"
            :title="collapsed ? '下载音乐' : undefined">
            <IconDownload class="w-[18px] h-[18px] flex-shrink-0" />
            <span class="nav-label" :class="labelClass">下载音乐</span>
          </router-link>
          <router-link v-if="userStore.isLoggedIn" to="/cloud-music"
            :class="navItemClass"
            active-class="!text-content !bg-muted"
            :title="collapsed ? '音乐云盘' : undefined">
            <IconCloud class="w-[18px] h-[18px] flex-shrink-0" />
            <span class="nav-label" :class="labelClass">音乐云盘</span>
          </router-link>
        </div>
      </div>

      <!-- 我的歌单 -->
      <div v-if="userStore.isLoggedIn" class="mt-4 mb-1 pt-2">
        <!-- 展开模式：内联歌单列表 -->
        <template v-if="!collapsed">
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
        </template>

        <!-- 折叠模式：图标 + Teleport popover 浮层 -->
        <template v-else>
          <div class="border-t border-line-2/50 my-2"></div>
          <div
            ref="createdTriggerRef"
            class="relative"
            @mouseenter="openPopover('created')"
            @mouseleave="scheduleClose('created')"
          >
            <div :class="navItemClass" class="cursor-pointer" title="我的歌单">
              <IconListMusic class="w-[18px] h-[18px] flex-shrink-0" />
            </div>
          </div>
        </template>
      </div>

      <!-- 收藏的歌单 -->
      <div v-if="userStore.isLoggedIn" class="mt-4 mb-1 pt-2">
        <template v-if="!collapsed">
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
        </template>

        <template v-else>
          <div
            ref="subTriggerRef"
            class="relative"
            @mouseenter="openPopover('sub')"
            @mouseleave="scheduleClose('sub')"
          >
            <div :class="navItemClass" class="cursor-pointer" title="收藏的歌单">
              <IconStar class="w-[18px] h-[18px] flex-shrink-0" />
            </div>
          </div>
        </template>
      </div>

    <!-- 折叠模式歌单浮层（Teleport 到 body，避免被 overflow 裁剪） -->
    <Teleport to="body">
      <div
        v-if="popover.open && popover.type === 'created'"
        class="playlist-popover-portal flex flex-col"
        :style="popoverStyle"
        @mouseenter="cancelClose()"
        @mouseleave="scheduleClose('created')"
      >
        <p class="text-xs text-content-3 px-3 py-2 border-b border-line-2/40 flex-shrink-0">我的歌单</p>
        <div class="flex-1 min-h-0 overflow-y-auto py-1">
          <div v-for="pl in createdPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
            class="flex items-center gap-2.5 px-3 py-1.5 hover:bg-muted cursor-pointer transition">
            <img :src="pl.coverImgUrl + '?param=80y80'" class="w-8 h-8 rounded object-cover flex-shrink-0" />
            <span class="text-sm truncate text-content-2">{{ pl.name }}</span>
          </div>
          <div v-if="!createdPlaylists.length" class="px-3 py-2 text-xs text-content-3">暂无歌单</div>
        </div>
      </div>
      <div
        v-if="popover.open && popover.type === 'sub'"
        class="playlist-popover-portal flex flex-col"
        :style="popoverStyle"
        @mouseenter="cancelClose()"
        @mouseleave="scheduleClose('sub')"
      >
        <p class="text-xs text-content-3 px-3 py-2 border-b border-line-2/40 flex-shrink-0">收藏的歌单</p>
        <div class="flex-1 min-h-0 overflow-y-auto py-1">
          <div v-for="pl in subPlaylists" :key="pl.id" @click="goPlaylist(pl.id)"
            class="flex items-center gap-2.5 px-3 py-1.5 hover:bg-muted cursor-pointer transition">
            <img :src="pl.coverImgUrl + '?param=80y80'" class="w-8 h-8 rounded object-cover flex-shrink-0" />
            <span class="text-sm truncate text-content-2">{{ pl.name }}</span>
          </div>
          <div v-if="!subPlaylists.length" class="px-3 py-2 text-xs text-content-3">暂无歌单</div>
        </div>
      </div>
    </Teleport>

      <!-- 底部：设置 + 用户 -->
      <div class="mt-auto pt-4" :class="player.currentSong ? 'pb-20' : 'pb-2'">
        <div :class="collapsed ? 'px-0' : 'px-1'">
          <router-link to="/settings"
            :class="navItemClass"
            active-class="!text-content !bg-muted"
            :title="collapsed ? '设置' : undefined">
            <IconSettings class="w-[18px] h-[18px] flex-shrink-0" />
            <span class="nav-label" :class="labelClass">设置</span>
          </router-link>
        </div>

        <!-- 未登录 -->
        <div v-if="!userStore.isLoggedIn && !collapsed" class="mt-3 p-3 rounded-xl bg-subtle/60">
          <p class="text-xs text-content-3 mb-2">强烈建议登录以提升体验</p>
          <router-link to="/login"
            class="flex items-center justify-center gap-2 w-full px-4 py-2 rounded-lg bg-accent hover:bg-accent-hover transition text-sm font-medium text-white">
            <IconLogIn class="w-4 h-4" />
            立即登录
          </router-link>
        </div>
        <div v-else-if="!userStore.isLoggedIn && collapsed" class="mt-3 flex justify-center">
          <router-link to="/login"
            class="w-9 h-9 flex items-center justify-center rounded-lg bg-accent hover:bg-accent-hover transition text-white"
            title="登录">
            <IconLogIn class="w-4 h-4" />
          </router-link>
        </div>

        <!-- 已登录 -->
        <div v-else-if="!collapsed" class="flex items-center gap-3 px-2 mt-3">
          <img :src="userStore.user?.avatarUrl" class="w-8 h-8 rounded-full ring-2 ring-accent/50" />
          <div class="min-w-0">
            <p class="text-sm font-medium truncate">{{ userStore.user?.nickname }}</p>
            <button @click="userStore.logout(); player.stop()"
              class="text-xs text-content-3 hover:text-danger transition">退出登录</button>
          </div>
        </div>
        <div v-else class="mt-3 flex justify-center">
          <img :src="userStore.user?.avatarUrl"
            class="w-8 h-8 rounded-full ring-2 ring-accent/50 cursor-pointer"
            :title="userStore.user?.nickname"
            @click="router.push('/settings')" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed, reactive } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useUserStore } from '../stores/user';
import { usePlayerStore } from '../stores/player';
import { useUiStore } from '../stores/ui';
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
import IconListMusic from '~icons/lucide/list-music';
import IconStar from '~icons/lucide/star';

const props = defineProps<{
  collapsed: boolean;
}>();

const emit = defineEmits<{
  navigate: [];
}>();

const router = useRouter();
const route = useRoute();
const userStore = useUserStore();
const player = usePlayerStore();
const ui = useUiStore();

// 导航项样式：始终 flex 布局，折叠时居中无文字宽度
const navItemClass = computed(() => [
  'flex items-center rounded-lg transition-colors duration-200 text-content-2 hover:text-content hover:bg-subtle',
  props.collapsed
    ? 'w-9 h-9 mx-auto justify-center'
    : 'gap-3 px-3 py-2 w-full',
]);

// 文字标签样式：用 max-width + overflow-hidden 平滑过渡，防止换行乱跳
const labelClass = computed(() => [
  'nav-label whitespace-nowrap overflow-hidden transition-all duration-300',
  props.collapsed ? 'max-w-0 opacity-0' : 'max-w-[160px] opacity-100',
]);

const createdPlaylists = ref<any[]>([]);
const subPlaylists = ref<any[]>([]);
const showCreatedPlaylists = ref(true);
const showSubPlaylists = ref(true);

// 折叠模式 popover 状态（Teleport 到 body，避免被 overflow 裁剪）
const createdTriggerRef = ref<HTMLElement | null>(null);
const subTriggerRef = ref<HTMLElement | null>(null);
const popover = reactive({ open: false, type: 'created' as 'created' | 'sub' });
let closeTimer: ReturnType<typeof setTimeout> | null = null;

const popoverStyle = computed(() => {
  const el = popover.type === 'created' ? createdTriggerRef.value : subTriggerRef.value;
  if (!el) return { display: 'none' };
  const rect = el.getBoundingClientRect();
  // 浮层可用最大高度：从触发点顶部到 playerBar 顶部
  // playerBar 高度 64px (h-16) + 8px 间距；无播放时仅留 8px 底部边距
  // 避免浮层底部被 playerBar 遮挡，看不到下方歌单
  const playerBarReserved = player.currentSong ? 72 : 8;
  const maxH = Math.max(120, window.innerHeight - rect.top - playerBarReserved);
  return {
    position: 'fixed' as const,
    left: `${rect.right + 8}px`,
    top: `${rect.top}px`,
    maxHeight: `${maxH}px`,
  };
});

function openPopover(type: 'created' | 'sub') {
  if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
  popover.type = type;
  popover.open = true;
}
function scheduleClose(_type: 'created' | 'sub') {
  if (closeTimer) clearTimeout(closeTimer);
  closeTimer = setTimeout(() => { popover.open = false; }, 200);
}
function cancelClose() {
  if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
}

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
  popover.open = false;
  router.push({ name: 'playlist', params: { id } });
  emit('navigate');
}

function isPlaylistActive(id: number): boolean {
  return route.name === 'playlist' && Number(route.params.id) === id;
}

async function openRoamFromSidebar() {
  if (!userStore.isLoggedIn) {
    router.push('/login');
    emit('navigate');
    return;
  }
  if (player.isFmMode) {
    ui.openRoamDrawer();
  } else {
    await player.loadFm();
  }
  emit('navigate');
}

// 路由变化时通知父组件（抽屉模式下关闭抽屉）
watch(() => route.fullPath, () => {
  emit('navigate');
});

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

<style scoped>
/* 折叠模式歌单浮层样式（Teleport 到 body，需用全局类名） */
</style>

<style>
.playlist-popover-portal {
  min-width: 200px;
  background-color: var(--color-surface, #fff);
  border: 1px solid var(--color-line-2, rgba(0,0,0,0.1));
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  z-index: 9999;
  overflow: hidden;
  animation: popoverFadeIn 0.2s ease;
}
@keyframes popoverFadeIn {
  from { opacity: 0; transform: translateX(-8px); }
  to { opacity: 1; transform: translateX(0); }
}
</style>
