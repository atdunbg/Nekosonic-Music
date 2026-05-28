import { createApp } from 'vue';
import App from './App.vue';
import './style.css';
import router from './router';
import { createPinia } from 'pinia';

const preventGesture = (e: Event) => e.preventDefault();
document.addEventListener('gesturestart', preventGesture, { passive: false });
document.addEventListener('gesturechange', preventGesture, { passive: false });
document.addEventListener('gestureend', preventGesture, { passive: false });

window.addEventListener('wheel', (e: WheelEvent) => {
  if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
    e.preventDefault();
  }
}, { passive: false });

window.addEventListener('touchmove', (e: TouchEvent) => {
  if (e.touches.length >= 2) {
    e.preventDefault();
  }
}, { passive: false });

const app = createApp(App);
app.use(router);
app.use(createPinia());
app.mount('#app');