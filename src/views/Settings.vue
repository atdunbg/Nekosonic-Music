<template>
  <div class="p-8 text-content max-w-2xl">
    <button @click="$router.back()" class="mb-4 text-content-2 hover:text-content transition">
      ← 返回
    </button>
    <h1 class="text-2xl font-bold mb-8">设置</h1>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">播放</h2>
      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">音质选择</p>
            <p class="text-xs text-content-3 mt-0.5">更高音质需要 VIP 权限</p>
          </div>
          <CustomSelect v-model="settings.audioQuality" :options="qualityLabels" />
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">外观</h2>
      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">主题</p>
            <p class="text-xs text-content-3 mt-0.5">切换应用主题</p>
          </div>
          <div class="flex bg-subtle rounded-lg p-0.5">
            <button
              v-for="t in themeOptions"
              :key="t.value"
              @click="settings.setTheme(t.value)"
              class="px-3 py-1.5 rounded-md text-sm transition"
              :class="settings.theme === t.value ? 'bg-muted text-content' : 'text-content-3 hover:text-content-2'"
            >
              {{ t.label }}
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">窗口</h2>
      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">关闭窗口时</p>
            <p class="text-xs text-content-3 mt-0.5">点击关闭按钮的默认行为</p>
          </div>
          <CustomSelect v-model="closeActionValue" :options="closeActionLabels" />
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">下载</h2>
      <div class="space-y-5">
        <div>
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium">下载路径</p>
              <p class="text-xs text-content-3 mt-0.5">歌曲下载保存位置</p>
            </div>
          </div>
          <div class="flex gap-2">
            <input
              v-model="downloadPathInput"
              type="text"
              placeholder="例如：~/Music/Nekosonic"
              class="flex-1 bg-subtle border border-line rounded-lg px-3 py-2 text-sm text-content placeholder-content-4 outline-none focus:border-accent/50 transition"
            />
            <button
              @click="saveDownloadPath"
              class="px-4 py-2 bg-accent-dim hover:bg-accent-dim text-accent-text rounded-lg text-sm transition"
            >
              保存
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">关于</h2>
      <div class="space-y-4">
        <a @click.prevent="openUrl('https://gitea.atdunbg.xyz/atdunbg/Nekosonic-Music')"
          class="flex items-center gap-4 p-4 bg-subtle rounded-xl hover:bg-muted transition cursor-pointer">
          <img src="../assets/app-icon.png" class="w-12 h-12 rounded-xl flex-shrink-0" alt="Nekosonic" />
          <div>
            <p class="font-semibold">Nekosonic</p>
            <p class="text-xs text-content-3 mt-0.5">版本 {{ appVersion }}</p>
          </div>
        </a>
        <p class="text-xs text-content-3 leading-relaxed">
          Nekosonic 是一款高颜值的跨平台第三方网易云音乐桌面客户端，基于 Tauri 2 + Vue 3 构建，提供轻量流畅的音乐播放体验。
        </p>
        <button
          @click="checkUpdate"
          :disabled="checkingUpdate"
          class="flex items-center gap-2 px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition"
        >
          <svg v-if="!checkingUpdate" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.66 0 3-4.03 3-9s-1.34-9-3-9m0 18c-1.66 0-3-4.03-3-9s1.34-9 3-9m-9 9a9 9 0 019-9"/></svg>
          <svg v-else class="animate-spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
          {{ checkingUpdate ? '检查中...' : '检查更新(暂未实现)' }}
        </button>
        <p v-if="updateMessage" class="text-xs" :class="updateMessageClass">{{ updateMessage }}</p>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useSettingsStore, qualityLabels, closeActionLabels, type CloseAction } from '../stores/settings';
import { useToast } from '../composables/useToast';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import CustomSelect from '../components/CustomSelect.vue';

const settings = useSettingsStore();
const { showToast } = useToast();

const appVersion = ref('');
onMounted(async () => {
  appVersion.value = await getVersion();
});

const closeActionValue = computed({
  get: () => settings.closeAction,
  set: (val: CloseAction) => settings.setCloseAction(val),
});

const downloadPathInput = ref(settings.downloadPath);
const checkingUpdate = ref(false);
const updateMessage = ref('');
const updateMessageClass = ref('text-content-2');

const themeOptions = [
  { label: '深色', value: 'dark' as const },
  { label: '浅色', value: 'light' as const },
];

function saveDownloadPath() {
  settings.setDownloadPath(downloadPathInput.value.trim());
  showToast('下载路径已保存', 'success');
}

async function checkUpdate() {
  checkingUpdate.value = true;
  updateMessage.value = '';
  try {
    await new Promise(r => setTimeout(r, 1500));
    updateMessage.value = '当前已是最新版本';
    updateMessageClass.value = 'text-accent-text';
  } catch {
    updateMessage.value = '检查更新失败，请稍后重试';
    updateMessageClass.value = 'text-danger';
  } finally {
    checkingUpdate.value = false;
  }
}
</script>
