import { createApp } from 'vue';
import App from './App.vue';
import './style.css';
import router from './router';
import { createPinia } from 'pinia';

// ---------- 彻底阻止双指拖动和手势 ----------
const preventGesture = (e: Event) => e.preventDefault();

// 阻止 iOS / macOS 手势缩放和页面拖动
document.addEventListener('gesturestart', preventGesture, { passive: false });
document.addEventListener('gesturechange', preventGesture, { passive: false });
document.addEventListener('gestureend', preventGesture, { passive: false });

// 阻止触控板双指水平滑动（若仍存在）
window.addEventListener('wheel', (e: WheelEvent) => {
  // 只阻止水平方向，保留垂直滚动（内部容器会处理）
  if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
    e.preventDefault();
  }
}, { passive: false });

// 阻止移动端双指触摸移动（不影响单指滚动）
window.addEventListener('touchmove', (e: TouchEvent) => {
  if (e.touches.length >= 2) {
    e.preventDefault();
  }
}, { passive: false });
// -------------------------------------------

const app = createApp(App);
app.use(router);
app.use(createPinia());
app.mount('#app');