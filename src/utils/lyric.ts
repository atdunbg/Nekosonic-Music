export interface LyricLine {
  time: number;
  text: string;
  translation?: string;
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
  result.sort((a, b) => a.time - b.time);
  return result;
}

export function mergeTranslation(lyrics: LyricLine[], tLrcStr: string): LyricLine[] {
  if (!tLrcStr) return lyrics;
  const tLines = parseLrc(tLrcStr);
  if (tLines.length === 0) return lyrics;

  const tMap = new Map<number, string>();
  for (const t of tLines) {
    const key = Math.round(t.time * 100);
    tMap.set(key, t.text);
  }

  return lyrics.map(line => {
    const key = Math.round(line.time * 100);
    const translation = tMap.get(key);
    if (translation) {
      return { ...line, translation };
    }
    for (let offset = -3; offset <= 3; offset++) {
      const t = tMap.get(key + offset);
      if (t) {
        return { ...line, translation: t };
      }
    }
    return line;
  });
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
