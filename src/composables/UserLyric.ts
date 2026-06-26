import { watch } from 'vue';
import { usePlayerStore } from '../stores/player';
import { useLyricManager } from './useLyricManager';

/**
 * 歌词 composable（向后兼容入口）
 *
 * 委托给 useLyricManager，并自动绑定 player store 的 currentSong/currentTime。
 * 新代码建议直接使用 useLyricManager。
 */
export function useLyric() {
  const player = usePlayerStore();
  const manager = useLyricManager();

  // 跟随当前歌曲加载歌词
  watch(() => player.currentSong, (song) => {
    manager.loadLyric(song);
    // 预加载队列中下一首的歌词
    const next = player.queue[player.currentIndex + 1];
    if (next && next.id !== song?.id) {
      manager.preloadLyric(next);
    }
  }, { immediate: true });

  // 跟随播放时间更新当前歌词行
  watch(() => player.currentTime, (t) => {
    manager.updateCurrentIndex(t);
  });

  return {
    lyrics: manager.lyrics,
    currentLyricIdx: manager.currentLyricIdx,
    hasTranslation: manager.hasTranslation,
    showTranslation: manager.showTranslation,
    toggleTranslation: manager.toggleTranslation,
  };
}
