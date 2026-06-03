<template>
  <!-- 普通头部：随内容滚动，返回独占一行，标题和按钮在第二行 -->
  <div ref="headerRef" class="-mx-8 px-8 pt-3 pb-2">
    <button @click="$router.back()" class="mb-1 text-content-2 hover:text-content transition text-sm">
      ← 返回
    </button>
    <div class="flex items-center gap-4">
      <slot />
      <slot name="actions" />
    </div>
  </div>
  <!-- 粘性操作栏：滚动后显示，只有返回+小功能按钮 -->
  <div class="sticky top-0 z-10 -mx-8 px-8 py-1.5">
    <div
      class="absolute inset-0 backdrop-blur-md transition-opacity duration-300"
      :class="isStuck ? 'opacity-100' : 'opacity-0'"
      style="mask-image: linear-gradient(to bottom, black 0%, black 50%, transparent 100%); -webkit-mask-image: linear-gradient(to bottom, black 0%, black 50%, transparent 100%);"
    >
      <div class="w-full h-full bg-base/80"></div>
    </div>
    <div
      class="relative flex items-center gap-2 transition-opacity duration-300"
      :class="isStuck ? 'opacity-100' : 'opacity-0 pointer-events-none'"
    >
      <button @click="$router.back()" class="text-content-2 hover:text-content transition text-sm">
        ← 返回
      </button>
      <div class="flex-1" />
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const headerRef = ref<HTMLElement | null>(null)
const isStuck = ref(false)

let observer: IntersectionObserver | null = null

onMounted(() => {
  if (!headerRef.value) return
  observer = new IntersectionObserver(
    ([entry]) => {
      isStuck.value = !entry.isIntersecting
    },
    { threshold: 0 }
  )
  observer.observe(headerRef.value)
})

onUnmounted(() => {
  observer?.disconnect()
})
</script>
