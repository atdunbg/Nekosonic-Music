import { getCurrentWindow } from '@tauri-apps/api/window';

// 窗口控制：最小化 / 最大化切换
// TitleBar 与 TopBar 共用，避免重复实现
const currentWindow = getCurrentWindow();

export function minimizeWindow() {
  currentWindow.minimize();
}

export async function toggleMaximize() {
  const isMaximized = await currentWindow.isMaximized();
  if (isMaximized) {
    currentWindow.unmaximize();
  } else {
    currentWindow.maximize();
  }
}
