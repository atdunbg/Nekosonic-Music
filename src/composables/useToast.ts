import { ref } from 'vue';

export interface Toast {
  id: number;
  message: string;
  type: 'success' | 'error' | 'info';
}

const toasts = ref<Toast[]>([]);
let nextId = 0;

const MAX_TOASTS = 3;
const DEDUP_WINDOW = 3000;
const recentMessages = new Map<string, number>();

export function showToast(message: string, type: 'success' | 'error' | 'info' = 'info', duration = 3000) {
  const key = `${type}:${message}`;
  const now = Date.now();
  const lastShown = recentMessages.get(key);
  if (lastShown && now - lastShown < DEDUP_WINDOW) return;
  recentMessages.set(key, now);
  setTimeout(() => { recentMessages.delete(key); }, DEDUP_WINDOW);

  if (toasts.value.length >= MAX_TOASTS) {
    toasts.value.shift();
  }

  const id = nextId++;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }, duration);
}

export function useToast() {
  return { toasts, showToast };
}
