export interface SkinColors {
  bg: string;
  surface: string;
  subtle: string;
  muted: string;
  emphasis: string;
  content: string;
  content2: string;
  content3: string;
  content4: string;
  line: string;
  line2: string;
  accent: string;
  accentHover: string;
  accentText: string;
  accentDim: string;
  danger: string;
  dangerDim: string;
  warning: string;
  info: string;
}

interface PresetSkin {
  id: string;
  name: string;
  preview: string;
  colors: SkinColors;
}

function darkSkin(accent: string, accentHover: string, accentText: string, name: string, id: string): PresetSkin {
  return {
    id,
    name,
    preview: accent,
    colors: {
      bg: '#02060c',
      surface: '#0a101a',
      subtle: `rgba(${hexToRgb(accent)}, 0.06)`,
      muted: `rgba(${hexToRgb(accent)}, 0.10)`,
      emphasis: `rgba(${hexToRgb(accent)}, 0.18)`,
      content: '#ffffff',
      content2: '#9ca3af',
      content3: '#6b7280',
      content4: '#4b5563',
      line: 'rgba(255, 255, 255, 0.08)',
      line2: 'rgba(255, 255, 255, 0.04)',
      accent,
      accentHover,
      accentText,
      accentDim: `rgba(${hexToRgb(accent)}, 0.20)`,
      danger: '#ef4444',
      dangerDim: 'rgba(239, 68, 68, 0.20)',
      warning: '#eab308',
      info: '#8b5cf6',
    },
  };
}

function lightSkin(accent: string, accentHover: string, accentText: string, name: string, id: string): PresetSkin {
  return {
    id,
    name,
    preview: accent,
    colors: {
      bg: '#f8fafc',
      surface: '#ffffff',
      subtle: `rgba(${hexToRgb(accent)}, 0.06)`,
      muted: `rgba(${hexToRgb(accent)}, 0.10)`,
      emphasis: `rgba(${hexToRgb(accent)}, 0.18)`,
      content: '#0f172a',
      content2: '#475569',
      content3: '#94a3b8',
      content4: '#cbd5e1',
      line: 'rgba(0, 0, 0, 0.08)',
      line2: 'rgba(0, 0, 0, 0.04)',
      accent,
      accentHover,
      accentText,
      accentDim: `rgba(${hexToRgb(accent)}, 0.15)`,
      danger: '#ef4444',
      dangerDim: 'rgba(239, 68, 68, 0.15)',
      warning: '#eab308',
      info: '#8b5cf6',
    },
  };
}

function hexToRgb(hex: string): string {
  const h = hex.replace('#', '');
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  return `${r}, ${g}, ${b}`;
}

export const presetSkins: PresetSkin[] = [
  // 深色
  darkSkin('#3b82f6', '#2563eb', '#60a5fa', '深蓝', 'dark-blue'),
  darkSkin('#22c55e', '#16a34a', '#4ade80', '深翠', 'dark-green'),
  darkSkin('#f43f5e', '#e11d48', '#fb7185', '深红', 'dark-rose'),
  darkSkin('#8b5cf6', '#7c3aed', '#a78bfa', '深紫', 'dark-violet'),
  darkSkin('#f97316', '#ea580c', '#fb923c', '深橙', 'dark-orange'),
  darkSkin('#06b6d4', '#0891b2', '#22d3ee', '深青', 'dark-cyan'),
  darkSkin('#ec4899', '#db2777', '#f472b6', '深粉', 'dark-pink'),
  // 浅色
  lightSkin('#3b82f6', '#2563eb', '#2563eb', '浅蓝', 'light-blue'),
  lightSkin('#22c55e', '#16a34a', '#16a34a', '浅翠', 'light-green'),
  lightSkin('#f43f5e', '#e11d48', '#e11d48', '浅红', 'light-rose'),
  lightSkin('#8b5cf6', '#7c3aed', '#7c3aed', '浅紫', 'light-violet'),
  lightSkin('#f97316', '#ea580c', '#ea580c', '浅橙', 'light-orange'),
  lightSkin('#06b6d4', '#0891b2', '#0891b2', '浅青', 'light-cyan'),
  lightSkin('#ec4899', '#db2777', '#db2777', '浅粉', 'light-pink'),
];

const presetIdSet = new Set(presetSkins.map(s => s.id));

export function isPresetSkinId(id: string): boolean {
  return presetIdSet.has(id);
}

export function getPresetSkin(id: string): PresetSkin | undefined {
  return presetSkins.find(s => s.id === id);
}

/** 将皮肤颜色应用到 DOM CSS 变量 */
export function applySkinColors(colors: SkinColors) {
  const root = document.documentElement;
  const map: Record<keyof SkinColors, string> = {
    bg: '--c-bg',
    surface: '--c-surface',
    subtle: '--c-subtle',
    muted: '--c-muted',
    emphasis: '--c-emphasis',
    content: '--c-content',
    content2: '--c-content-2',
    content3: '--c-content-3',
    content4: '--c-content-4',
    line: '--c-line',
    line2: '--c-line-2',
    accent: '--c-accent',
    accentHover: '--c-accent-hover',
    accentText: '--c-accent-text',
    accentDim: '--c-accent-dim',
    danger: '--c-danger',
    dangerDim: '--c-danger-dim',
    warning: '--c-warning',
    info: '--c-info',
  };
  for (const [key, cssVar] of Object.entries(map)) {
    root.style.setProperty(cssVar, colors[key as keyof SkinColors]);
  }
}
