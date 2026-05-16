<template>
  <Transition name="fade">
    <div v-if="visible" class="fixed inset-0 z-[70] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="handleIgnore">
      <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[440px] max-h-[80vh] flex flex-col select-auto">
        <div class="p-6 pb-4">
          <div class="flex items-center gap-3 mb-1">
            <div class="w-10 h-10 rounded-xl bg-accent/15 flex items-center justify-center flex-shrink-0">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-accent-text"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            </div>
            <div>
              <h2 class="text-lg font-semibold text-content">发现新版本</h2>
              <p class="text-xs text-content-3 mt-0.5">
                v{{ info.currentVersion }} → <span class="text-accent-text font-medium">v{{ info.version }}</span>
              </p>
            </div>
          </div>
          <p v-if="info.date" class="text-xs text-content-3 mt-2 ml-[52px]">{{ formatDate(info.date) }}</p>
        </div>

        <div v-if="info.body" class="px-6 pb-4 flex-1 overflow-y-auto max-h-60 ml-[52px]">
          <div class="text-sm text-content-2 leading-relaxed whitespace-pre-wrap">{{ info.body }}</div>
        </div>
        <div v-else class="px-6 pb-4 ml-[52px]">
          <p class="text-sm text-content-3">暂无更新日志</p>
        </div>

        <div v-if="downloading" class="px-6 pb-2">
          <div class="w-full bg-subtle rounded-full h-2 overflow-hidden">
            <div class="h-full bg-accent rounded-full transition-all duration-300" :style="{ width: downloadProgress + '%' }"></div>
          </div>
          <p class="text-xs text-content-3 mt-1 text-center">正在下载更新 {{ downloadProgress }}%</p>
        </div>

        <div class="p-4 border-t border-line flex gap-3">
          <button
            @click="handleIgnore"
            :disabled="downloading"
            class="flex-1 py-2 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition disabled:opacity-50"
          >
            忽略此版本
          </button>
          <button
            @click="handleUpdate"
            :disabled="downloading"
            class="flex-1 py-2 rounded-lg bg-accent hover:bg-accent-hover text-white text-sm font-medium transition disabled:opacity-50"
          >
            {{ downloading ? `下载中 ${downloadProgress}%` : '立即更新' }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import type { UpdateInfo } from '../composables/useUpdater'

const props = defineProps<{
  visible: boolean
  info: UpdateInfo & { currentVersion: string }
  downloading: boolean
  downloadProgress: number
}>()

const emit = defineEmits<{
  update: []
  ignore: []
}>()

function handleUpdate() {
  emit('update')
}

function handleIgnore() {
  if (props.downloading) return
  emit('ignore')
}

function formatDate(dateStr: string) {
  try {
    const d = new Date(dateStr)
    return d.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })
  } catch {
    return dateStr
  }
}
</script>
