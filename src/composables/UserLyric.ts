import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { parseLrc, getCurrentLyricIndex, LyricLine } from '../utils/lyric';
import { usePlayerStore } from '../stores/player';

export function useLyric() {
  const player = usePlayerStore();

  const lyrics = ref<LyricLine[]>([]);
  const currentLyricIdx = ref(-1);

  watch(() => player.currentSong, async (song) => {
    if (!song) {
      lyrics.value = [];
      currentLyricIdx.value = -1;
      return;
    }
    try {
      const jsonStr: string = await invoke('get_lyric', { id: song.id });
      const data = JSON.parse(jsonStr);
      const lrc = data?.lrc?.lyric || '';
      lyrics.value = lrc ? parseLrc(lrc) : [];
      currentLyricIdx.value = -1;
    } catch {
      lyrics.value = [];
    }
  }, { immediate: true });

  watch(() => player.currentTime, (t) => {
    if (lyrics.value.length === 0) return;
    const idx = getCurrentLyricIndex(lyrics.value, t);
    if (idx !== currentLyricIdx.value) {
      currentLyricIdx.value = idx;
    }
  });

  return {
    lyrics,
    currentLyricIdx,
  };
}