<template>
  <div class="p-8 text-content">
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
          <div class="flex gap-2 items-center">
            <div class="flex-1 bg-subtle border border-line rounded-lg px-3 py-2 text-sm text-content-2 truncate" :title="settings.downloadPath || defaultDownloadPath">
              {{ settings.downloadPath || defaultDownloadPath }}
            </div>
            <button
              @click="pickDownloadFolder"
              class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 bg-accent/15 text-accent-text hover:bg-accent/25 active:scale-95"
            >
              选择文件夹
            </button>
            <button
              v-if="settings.downloadPath"
              @click="clearDownloadPath"
              class="px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200 bg-muted text-content-2 hover:bg-emphasis hover:text-content active:scale-95"
              title="重置为默认路径"
            >
              重置
            </button>
          </div>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">快捷键</h2>
      <div class="space-y-3">
        <div
          v-for="(sc, id) in settings.shortcuts"
          :key="id"
          class="flex items-center justify-between p-3 bg-subtle rounded-xl"
        >
          <div>
            <p class="text-sm font-medium">{{ sc.label }}</p>
          </div>
          <div class="flex items-center gap-1.5">
            <button
              v-if="sc.key !== defaultShortcuts[id]?.key"
              @click="settings.setShortcut(id, defaultShortcuts[id].key)"
              class="w-6 h-6 flex items-center justify-center rounded-md text-content-4 hover:text-danger hover:bg-danger/10 transition"
              title="恢复默认"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
            <button
              @click="startRecording(id)"
              class="px-3 py-1.5 rounded-lg text-sm transition min-w-[120px] text-center"
              :class="recordingId === id ? 'bg-accent text-white' : 'bg-muted hover:bg-emphasis text-content-2'"
            >
              {{ recordingId === id ? '按下新快捷键...' : formatShortcut(sc.key) }}
            </button>
          </div>
        </div>
        <button
          @click="resetShortcuts"
          class="text-xs text-content-3 hover:text-danger transition"
        >
          恢复默认快捷键
        </button>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">其他</h2>
      <div class="space-y-3">
        <div class="flex items-center justify-between p-3 bg-subtle rounded-xl">
          <div>
            <p class="text-sm font-medium">恢复默认设置</p>
            <p class="text-xs text-content-3 mt-0.5">重置所有设置为初始状态</p>
          </div>
          <button
            @click="handleResetAll"
            class="px-3 py-1.5 rounded-lg text-sm bg-muted hover:bg-emphasis text-danger transition"
          >
            重置
          </button>
        </div>
      </div>
    </section>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">关于</h2>
      <div class="space-y-4">
        <a @click.prevent="openUrl('https://github.com/atdunbg/Nekosonic-Music')"
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
        <div class="flex items-center gap-2">
          <button
            @click="handleCheckUpdate"
            :disabled="updater.checking.value"
            class="flex items-center gap-2 px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition"
          >
            <svg v-if="!updater.checking.value" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
            <svg v-else class="animate-spin" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-6.22-8.56"/></svg>
            {{ updater.checking.value ? '检查中...' : '检查更新' }}
          </button>
          <button
            @click="fetchChangelog"
            :disabled="fetchingChangelog"
            class="flex items-center gap-2 px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
            {{ fetchingChangelog ? '获取中...' : '更新日志' }}
          </button>
        </div>
        <p v-if="updater.error.value" class="text-xs text-content-3">{{ updater.error.value }}</p>
      </div>
    </section>
    <Transition name="fade">
      <div v-if="showResetConfirm" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showResetConfirm = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-80 p-6 select-auto">
          <h2 class="text-lg font-semibold text-content mb-1">确认重置</h2>
          <p class="text-sm text-content-2 mb-5">所有设置将恢复为默认值，此操作不可撤销。</p>
          <div class="flex gap-3">
            <button @click="showResetConfirm = false"
              class="flex-1 py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">
              取消
            </button>
            <button @click="confirmResetAll"
              class="flex-1 py-2 rounded-lg bg-danger/20 hover:bg-danger/30 text-danger text-sm font-medium transition">
              确认重置
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="fade">
      <div v-if="showChangelogModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showChangelogModal = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[480px] max-h-[80vh] flex flex-col select-auto">
          <div class="p-6 pb-4">
            <div class="flex items-center justify-between mb-1">
              <h2 class="text-lg font-semibold text-content">更新日志</h2>
              <span v-if="changelogRelease" class="text-xs font-medium px-2 py-0.5 rounded-full bg-accent/15 text-accent-text">v{{ changelogRelease.tag_name?.replace('v', '') }}</span>
            </div>
            <p v-if="changelogRelease?.published_at" class="text-xs text-content-3 mt-1">{{ formatDate(changelogRelease.published_at) }}</p>
          </div>
          <div v-if="changelogRelease?.body" class="px-6 pb-4 flex-1 overflow-y-auto max-h-60">
            <div class="text-sm text-content-2 leading-relaxed whitespace-pre-wrap">{{ changelogRelease.body }}</div>
          </div>
          <div v-else class="px-6 pb-4">
            <p class="text-sm text-content-3">暂无更新日志</p>
          </div>
          <div class="p-4 border-t border-line flex gap-3">
            <button @click="showChangelogModal = false"
              class="flex-1 py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">
              关闭
            </button>
            <button v-if="changelogRelease?.html_url" @click="openUrl(changelogRelease.html_url)"
              class="flex-1 py-2 rounded-lg bg-accent/20 hover:bg-accent/30 text-accent-text text-sm font-medium transition">
              在 GitHub 中查看
            </button>
          </div>
        </div>
      </div>
    </Transition>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useSettingsStore, qualityLabels, closeActionLabels, defaultShortcuts, type CloseAction } from '../stores/settings';
import { useToast } from '../composables/useToast';
import { useUpdater } from '../composables/useUpdater';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { open } from '@tauri-apps/plugin-dialog';
import CustomSelect from '../components/CustomSelect.vue';

const settings = useSettingsStore();
const { showToast } = useToast();
const updater = useUpdater();

const appVersion = ref('');
const defaultDownloadPath = ref('');
onMounted(async () => {
  appVersion.value = await getVersion();
  try {
    defaultDownloadPath.value = await invoke<string>('get_default_download_path');
  } catch {}
});

const closeActionValue = computed({
  get: () => settings.closeAction,
  set: (val: CloseAction) => settings.setCloseAction(val),
});

async function pickDownloadFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择下载路径',
  });
  if (selected) {
    settings.setDownloadPath(selected);
    showToast('下载路径已更新', 'success');
  }
}

function clearDownloadPath() {
  settings.setDownloadPath('');
  showToast('已重置为默认路径', 'success');
}

const themeOptions = [
  { label: '深色', value: 'dark' as const },
  { label: '浅色', value: 'light' as const },
];

async function handleCheckUpdate() {
  const result = await updater.checkForUpdate(false);
  if (!result) {
    showToast(updater.error.value || '当前已是最新版本', 'info');
  }
}

const fetchingChangelog = ref(false);
const changelogRelease = ref<any>(null);
const showChangelogModal = ref(false);

async function fetchChangelog() {
  fetchingChangelog.value = true;
  try {
    const resp = await fetch('https://api.github.com/repos/atdunbg/Nekosonic-Music/releases?per_page=1');
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const releases = await resp.json();
    if (releases && releases.length > 0) {
      changelogRelease.value = releases[0];
      showChangelogModal.value = true;
    } else {
      showToast('暂无发布版本', 'info');
    }
  } catch (e: any) {
    showToast(`获取失败: ${e}`, 'error');
  } finally {
    fetchingChangelog.value = false;
  }
}

function formatDate(dateStr: string) {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' });
  } catch {
    return dateStr;
  }
}

const recordingId = ref<string | null>(null);

function formatShortcut(key: string): string {
  return key
    .replace('Control', 'Ctrl')
    .replace('ArrowLeft', '←')
    .replace('ArrowRight', '→')
    .replace('ArrowUp', '↑')
    .replace('ArrowDown', '↓')
    .replace(/\+/g, ' + ');
}

function startRecording(id: string) {
  recordingId.value = id;
}

function resetShortcuts() {
  settings.resetShortcuts();
  showToast('快捷键已恢复默认', 'success');
}

const showResetConfirm = ref(false);

function handleResetAll() {
  showResetConfirm.value = true;
}

function confirmResetAll() {
  settings.resetAll();
  showResetConfirm.value = false;
  showToast('已恢复默认设置', 'success');
}

function onRecordingKeydown(e: KeyboardEvent) {
  if (!recordingId.value) return;
  e.preventDefault();
  e.stopPropagation();

  if (e.key === 'Escape') {
    recordingId.value = null;
    return;
  }

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('Control');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');

  const ignoredKeys = ['Control', 'Alt', 'Shift', 'Meta'];
  if (!ignoredKeys.includes(e.key)) {
    parts.push(e.code);
  }

  if (parts.length > 0 && !ignoredKeys.includes(e.key)) {
    const combo = parts.join('+');
    settings.setShortcut(recordingId.value, combo);
    recordingId.value = null;
  }
}

onMounted(() => {
  window.addEventListener('keydown', onRecordingKeydown, true);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onRecordingKeydown, true);
});
</script>
