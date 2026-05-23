import { ref, onMounted, onBeforeUnmount } from 'vue';

const isOnline = ref(navigator.onLine);

function update() {
  isOnline.value = navigator.onLine;
}

let refCount = 0;

export function useOnlineStatus() {
  onMounted(() => {
    refCount++;
    if (refCount === 1) {
      window.addEventListener('online', update);
      window.addEventListener('offline', update);
    }
  });

  onBeforeUnmount(() => {
    refCount--;
    if (refCount <= 0) {
      refCount = 0;
      window.removeEventListener('online', update);
      window.removeEventListener('offline', update);
    }
  });

  return { isOnline };
}
