<template>
  <!-- 详情页统一布局（参考 SPlayer list-detail 架构）
       absolute; inset:0 脱离 mainScrollRef 滚动流，钉死覆盖视口；
       内部 flex 列布局：header flex-shrink:0 常驻顶部，content flex-1 min-h:0 独立滚动。
       父级 mainScrollRef 已加 position:relative 作定位上下文。
       滚动 content 超过阈值时切换 is-compact（参考 SPlayer useListScroll），通过插槽 prop 透出。 -->
  <div class="detail-layout" :class="{ 'is-compact': isCompact }">
    <header class="detail-header">
      <slot name="header" :compact="isCompact" />
    </header>
    <div ref="contentRef" class="detail-content" data-scroll @scroll="onScroll">
      <slot name="content" :compact="isCompact" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount, nextTick } from 'vue';

defineOptions({ name: 'DetailLayout' });

const contentRef = ref<HTMLElement | null>(null);
// 紧凑态：参考 SPlayer useListScroll，scrollTop > 10 触发
const isCompact = ref(false);
const COMPACT_THRESHOLD = 10;
// 内容不足时不触发紧凑（避免短列表也收缩头部）
const MIN_SCROLL_RANGE = 150;

let ticking = false;
function onScroll() {
  if (ticking) return;
  ticking = true;
  requestAnimationFrame(() => {
    ticking = false;
    const el = contentRef.value;
    if (!el) return;
    // 内容不足以滚动时保持展开
    if (el.scrollHeight - el.clientHeight < MIN_SCROLL_RANGE) {
      if (isCompact.value) isCompact.value = false;
      return;
    }
    isCompact.value = el.scrollTop > COMPACT_THRESHOLD;
  });
}

// 供父组件重置滚动 & 紧凑态（切换页面/数据时）
function resetScroll() {
  isCompact.value = false;
  nextTick(() => {
    if (contentRef.value) contentRef.value.scrollTop = 0;
  });
}

defineExpose({ contentRef, isCompact, resetScroll });

onBeforeUnmount(() => {
  isCompact.value = false;
});
</script>

<style scoped>
/* 根容器：absolute 钉死填满 mainScrollRef 视口 + flex 列布局
   absolute 元素不撑高父级 → mainScrollRef 不滚动 → header 不被带走 */
.detail-layout {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 头部：flex-shrink:0 常驻顶部，不随内容滚动 */
.detail-header {
  flex-shrink: 0;
  /* 头部内部所有过渡元素统一动画曲线（参考 SPlayer transition: height/font-size 0.3s） */
  transition: height 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 内容区：flex-1 min-h:0 占满剩余空间，独立滚动 */
.detail-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
</style>
