<template>
  <!-- 普通头部：随内容滚动，标题和按钮一行 -->
  <div class="-mx-8 px-8 pt-3 pb-2">
    <div class="flex items-center gap-4">
      <slot />
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
</script>
