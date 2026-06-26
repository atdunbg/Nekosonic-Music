import { defineStore } from 'pinia';
import { ref } from 'vue';

/**
 * 侧边栏显示模式
 * - expanded：完整展开（224px，显示图标+文字）
 * - collapsed：折叠为图标栏（64px，仅图标）
 * - drawer：抽屉模式（窄屏时，覆盖式滑出）
 */
export type SidebarMode = 'expanded' | 'collapsed' | 'drawer';

/**
 * UI 状态 store
 *
 * 管理 Roam 抽屉（全屏播放/歌词/评论）和侧边栏相关的 UI 状态。
 */
export const useUiStore = defineStore('ui', () => {
  /** Roam 抽屉是否展开 */
  const showRoamDrawer = ref(false);
  /** 当前 Roam 抽屉显示的标签页 */
  const roamTab = ref<'lyric' | 'comment'>('lyric');
  /** Roam 抽屉评论区对应的歌曲 ID（null 表示使用 currentSong.id） */
  const commentSongId = ref<number | null>(null);
  /** 当前歌曲封面提取的主色调（用于沉浸式背景） */
  const dominantColor = ref('');

  /** 侧边栏模式（持久化到 localStorage） */
  const sidebarMode = ref<SidebarMode>(loadSidebarMode());
  /** 抽屉模式下侧边栏是否展开 */
  const drawerOpen = ref(false);

  function loadSidebarMode(): SidebarMode {
    const saved = localStorage.getItem('sidebar_mode');
    if (saved === 'expanded' || saved === 'collapsed') return saved;
    return 'expanded';
  }

  function setSidebarMode(mode: SidebarMode) {
    sidebarMode.value = mode;
    if (mode !== 'drawer') {
      localStorage.setItem('sidebar_mode', mode);
    }
  }

  /** 切换展开/折叠（不影响 drawer 模式） */
  function toggleSidebar() {
    if (sidebarMode.value === 'expanded') {
      setSidebarMode('collapsed');
    } else if (sidebarMode.value === 'collapsed') {
      setSidebarMode('expanded');
    }
  }

  /** 打开/关闭抽屉（仅 drawer 模式生效） */
  function toggleDrawer() {
    drawerOpen.value = !drawerOpen.value;
  }

  /** 关闭抽屉 */
  function closeDrawer() {
    drawerOpen.value = false;
  }

  /** 打开 Roam 抽屉，可指定初始标签页 */
  function openRoamDrawer(tab: 'lyric' | 'comment' = 'lyric') {
    roamTab.value = tab;
    showRoamDrawer.value = true;
  }

  /** 打开 Roam 抽屉并定位到指定歌曲的评论区 */
  function openCommentForSong(songId: number) {
    commentSongId.value = songId;
    openRoamDrawer('comment');
  }

  /** 关闭 Roam 抽屉并重置标签页 */
  function closeRoamDrawer() {
    showRoamDrawer.value = false;
    roamTab.value = 'lyric';
  }

  /** 切换 Roam 抽屉展开状态 */
  function toggleRoamDrawer() {
    showRoamDrawer.value = !showRoamDrawer.value;
  }

  return {
    showRoamDrawer,
    roamTab,
    commentSongId,
    dominantColor,
    sidebarMode,
    drawerOpen,
    setSidebarMode,
    toggleSidebar,
    toggleDrawer,
    closeDrawer,
    openRoamDrawer,
    openCommentForSong,
    closeRoamDrawer,
    toggleRoamDrawer,
  };
});
