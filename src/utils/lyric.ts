export interface LyricLine {
  time: number;  // 秒
  text: string;
}

export function parseLrc(lrcStr: string): LyricLine[] {
  const lines = lrcStr.split('\n');
  const result: LyricLine[] = [];
  const timeReg = /\[(\d{2}):(\d{2})\.(\d{2,3})\]/;
  for (const line of lines) {
    const match = line.match(timeReg);
    if (match) {
      const min = parseInt(match[1], 10);
      const sec = parseInt(match[2], 10);
      const ms = parseInt(match[3], 10) / (match[3].length === 3 ? 1000 : 100);
      const time = min * 60 + sec + ms;
      const text = line.replace(timeReg, '').trim();
      if (text) {
        result.push({ time, text });
      }
    }
  }
  // 按时长排序
  result.sort((a, b) => a.time - b.time);
  return result;
}

export function getCurrentLyricIndex(lyrics: LyricLine[], currentTime: number): number {
  let index = -1;
  for (let i = 0; i < lyrics.length; i++) {
    if (currentTime >= lyrics[i].time) {
      index = i;
    } else {
      break;
    }
  }
  return index;
}