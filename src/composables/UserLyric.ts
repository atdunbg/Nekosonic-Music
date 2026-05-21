import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { parseLrc, mergeTranslation, getCurrentLyricIndex, LyricLine } from '../utils/lyric';
import { usePlayerStore } from '../stores/player';

export function useLyric() {
  const player = usePlayerStore();

  const lyrics = ref<LyricLine[]>([]);
  const currentLyricIdx = ref(-1);
  const showTranslation = ref(true);
  const hasTranslation = ref(false);

  watch(() => player.currentSong, async (song) => {
    if (!song) {
      lyrics.value = [];
      currentLyricIdx.value = -1;
      hasTranslation.value = false;
      return;
    }
    try {
      const jsonStr: string = await invoke('get_lyric', { id: song.id });
      const data = JSON.parse(jsonStr);
      const lrc = data?.lrc?.lyric || '';
      const tLrc = data?.tlyric?.lyric || '';
      let parsed = lrc ? parseLrc(lrc) : [];
      if (tLrc && parsed.length > 0) {
        parsed = mergeTranslation(parsed, tLrc);
        hasTranslation.value = parsed.some(l => l.translation);
      } else {
        hasTranslation.value = false;
      }
      lyrics.value = parsed;
      currentLyricIdx.value = -1;
    } catch {
      lyrics.value = [];
      hasTranslation.value = false;
    }
  }, { immediate: true });

  watch(() => player.currentTime, (t) => {
    if (lyrics.value.length === 0) return;
    const idx = getCurrentLyricIndex(lyrics.value, t);
    if (idx !== currentLyricIdx.value) {
      currentLyricIdx.value = idx;
    }
  });

  function toggleTranslation() {
    showTranslation.value = !showTranslation.value;
  }

  return {
    lyrics,
    currentLyricIdx,
    hasTranslation,
    showTranslation,
    toggleTranslation,
  };
}
