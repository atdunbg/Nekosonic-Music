<template>
  <!-- 抽屉模式：遮罩 + 滑出面板 -->
  <template v-if="mode === 'drawer'">
    <Transition name="fade">
      <div v-if="ui.drawerOpen" class="fixed inset-0 bg-black/50 z-40" @click="ui.closeDrawer()"></div>
    </Transition>
    <Transition name="slide">
      <nav v-if="ui.drawerOpen" class="fixed left-0 top-0 bottom-0 w-64 z-50 flex flex-col shadow-2xl backdrop-blur-md" :style="sidebarBgStyle">
        <!-- Logo 区域 -->
        <div class="h-12 flex items-center flex-shrink-0 px-4 gap-2" data-tauri-drag-region>
          <IconMusic class="w-7 h-7 text-accent flex-shrink-0" />
          <span class="text-base font-bold text-content truncate">Nekosonic</span>
        </div>
        <SidebarContent :collapsed="false" @navigate="ui.closeDrawer()" />
      </nav>
    </Transition>
  </template>

  <!-- 展开/折叠模式：常驻侧栏（透明背景，靠右边框分隔） -->
  <nav
    v-else
    class="relative flex-shrink-0 flex flex-col transition-all duration-300 border-r border-line-2/40"
    :class="mode === 'collapsed' ? 'w-16' : 'w-56'"
  >
    <!-- Logo 区域（高度与 TopBar 一致，用于对齐） -->
    <div
      class="h-12 flex items-center flex-shrink-0"
      :class="mode === 'collapsed' ? 'justify-center' : 'px-4 gap-2'"
      data-tauri-drag-region
    >
      <IconMusic class="w-7 h-7 text-accent flex-shrink-0 cursor-pointer" @click="router.push('/')" />
      <span v-if="mode === 'expanded'" class="text-base font-bold text-content truncate cursor-pointer" @click="router.push('/')">Nekosonic</span>
    </div>

    <!-- 导航内容 -->
    <SidebarContent :collapsed="mode === 'collapsed'" />

    <!-- 折叠/展开触发器：复刻 naive-ui n-layout-toggle-bar（SPlayer 同款）
         两根竖条，中间 4px 重叠（无可见间隙），默认含蓄低饱和不抢聚焦；
         hover 时两根竖条各自旋转，在中间分开形成 < 或 > 箭头：
         展开态 → <（指向左，折叠方向）；折叠态 → >（指向右，展开方向）。
         命中区 44×88，加大 hover 容错；无背景色，仅靠竖条亮度变化反馈。 -->
    <button
      @click="ui.toggleSidebar()"
      class="sider-trigger"
      :class="mode === 'expanded' ? 'is-expanded' : 'is-collapsed'"
      :title="mode === 'expanded' ? '折叠侧边栏' : '展开侧边栏'"
      aria-label="切换侧边栏"
    >
      <span class="toggle-bar">
        <span class="bar bar-top"></span>
        <span class="bar bar-bottom"></span>
      </span>
    </button>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useUiStore, type SidebarMode } from '../stores/ui';
import { useSettingsStore } from '../stores/settings';
import SidebarContent from './SidebarContent.vue';
import IconMusic from '~icons/lucide/music';

const ui = useUiStore();
const settings = useSettingsStore();
const router = useRouter();

defineProps<{
  mode: SidebarMode;
}>();

// 侧边栏透明：无壁纸时继承主容器背景，有壁纸时透出壁纸
// 仅抽屉模式需要半透明背景 + 模糊（浮在主内容之上）
const sidebarBgStyle = computed(() => {
  if (settings.currentWallpaper.path) {
    return { backgroundColor: 'rgba(0, 0, 0, 0.2)' };
  }
  return {};
});
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.25s ease;
}
.slide-enter-from,
.slide-leave-to {
  transform: translateX(-100%);
}

/* 复刻 naive-ui n-layout-toggle-bar（SPlayer 同款）实现细节
   参考源码：naive-ui es/layout/src/styles/layout-sider.cssr.mjs
   关键点：
   1. 两根竖条 width:4 height:38 left:14，top=0 / top=34 → 中间 4px 重叠（无可见间隙）
   2. hover 时各自旋转 ±12deg + scale(1.15) + translateY(±2px)，
      两根在中间分开、两端反向旋转，拼出 < 或 > 箭头
   3. 展开态（左 sider 未折叠）→ <（指向左，折叠方向）
      折叠态 → >（指向右，展开方向）
   4. 默认低饱和（--c-content-3 + opacity），hover 变亮；无背景色
   5. 命中区 44×88，加大 hover 容错 */
.sider-trigger {
  position: absolute;
  top: 50%;
  right: -22px;
  transform: translateY(-50%);
  z-index: 30;
  width: 44px;
  height: 88px;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
}

/* 双竖条容器：居中于命中区（尺寸沿用 naive-ui 32×72 比例，整体放大到 44×88）
   透明度加在容器上而非每根竖条上：两根实色竖条重叠时颜色不会叠加变深，
   整体随容器统一淡入淡出（参考 SPlayer .n-layout-toggle-bar { opacity:.3; &:hover{opacity:1} }） */
.toggle-bar {
  position: relative;
  width: 44px;
  height: 88px;
  opacity: 0.5;
  transition: opacity 0.3s ease;
}
.sider-trigger:hover .toggle-bar {
  opacity: 1;
}

/* 单根竖条：4×38，圆角，水平居中于 44 宽容器（left:20）
   实色背景（无自身 opacity），两根垂直排列，中间 4px 重叠（无可见间隙）：
   总高度 = 38 + 38 - 4 = 72，在 88 容器中垂直居中 → 起始 top = (88-72)/2 = 8
   bar-top: top 8（占 8..46）；bar-bottom: top 42（占 42..80），重叠区 42..46 */
.bar {
  position: absolute;
  width: 4px;
  height: 38px;
  left: 20px;
  border-radius: 2px;
  background-color: var(--c-content-3);
  transition:
    transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    background-color 0.3s ease;
}
.bar-top { top: 8px; }
.bar-bottom { top: 42px; }

/* hover 时两根竖条变亮（透明度由容器统一处理） */
.sider-trigger:hover .bar {
  background-color: var(--c-content);
}

/* 展开态 hover → < 箭头（指向左，折叠方向）
   naive-ui 公式：top = rotate(12deg) scale(1.15) translateY(-2px)
                  bottom = rotate(-12deg) scale(1.15) translateY(2px)
   两根中间端反向旋转分开、远端继续向外，形成 < */
.is-expanded:hover .bar-top {
  transform: rotate(12deg) scale(1.15) translateY(-2px);
}
.is-expanded:hover .bar-bottom {
  transform: rotate(-12deg) scale(1.15) translateY(2px);
}

/* 折叠态 hover → > 箭头（指向右，展开方向）
   naive-ui 公式：top = rotate(-12deg) scale(1.15) translateY(-2px)
                  bottom = rotate(12deg) scale(1.15) translateY(2px) */
.is-collapsed:hover .bar-top {
  transform: rotate(-12deg) scale(1.15) translateY(-2px);
}
.is-collapsed:hover .bar-bottom {
  transform: rotate(12deg) scale(1.15) translateY(2px);
}
</style>
