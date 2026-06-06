<template>
  <!-- 普通头部：随内容滚动，返回独占一行，标题和按钮在第二行 -->
  <div class="-mx-8 px-8 pt-3 pb-2">
    <button @click="$router.back()" class="mb-1 text-content-2 hover:text-content transition text-sm">
      ← 返回
    </button>
    <div class="flex items-center gap-4">
      <slot />
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
</script>
