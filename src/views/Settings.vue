<template>
  <div class="p-8 text-content">
    <PageHeader>
      <h1 class="text-2xl font-bold">设置</h1>
    </PageHeader>

    <section class="mb-8">
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">播放</h2>
      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium">输出设备</p>
            <p class="text-xs text-content-3 mt-0.5">选择音频播放设备</p>
          </div>
          <CustomSelect v-model="selectedDevice" :options="deviceOptions" />
        </div>

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
      <h2 class="text-sm text-content-2 uppercase tracking-wider mb-4">皮肤</h2>
      <div class="space-y-4">
        <!-- 明暗切换 -->
        <div class="flex gap-2">
          <button
            @click="settings.setSkin(toSkinId(currentThemeColor, 'dark'))"
            class="flex items-center gap-2 px-4 py-2 rounded-xl transition-all border-2"
            :class="!settings.isPreset || currentAppearance !== 'dark' ? 'border-transparent bg-subtle hover:bg-muted' : 'border-accent/40 bg-accent/10'"
          >
            <IconMoon class="w-4 h-4" :class="currentAppearance === 'dark' && settings.isPreset ? 'text-accent-text' : 'text-content-3'" />
            <span class="text-sm" :class="currentAppearance === 'dark' && settings.isPreset ? 'text-content font-medium' : 'text-content-3'">深色</span>
          </button>
          <button
            @click="settings.setSkin(toSkinId(currentThemeColor, 'light'))"
            class="flex items-center gap-2 px-4 py-2 rounded-xl transition-all border-2"
            :class="!settings.isPreset || currentAppearance !== 'light' ? 'border-transparent bg-subtle hover:bg-muted' : 'border-accent/40 bg-accent/10'"
          >
            <IconSun class="w-4 h-4" :class="currentAppearance === 'light' && settings.isPreset ? 'text-accent-text' : 'text-content-3'" />
            <span class="text-sm" :class="currentAppearance === 'light' && settings.isPreset ? 'text-content font-medium' : 'text-content-3'">浅色</span>
          </button>
        </div>

        <!-- 主题色选择 -->
        <div class="grid grid-cols-7 gap-2">
          <div
            v-for="tc in themeColorOptions"
            :key="tc.id"
            @click="settings.setSkin(toSkinId(tc.id, currentAppearance))"
            class="flex flex-col items-center gap-1.5 p-2 rounded-xl transition-all border-2 cursor-pointer"
            :class="currentThemeColor === tc.id && settings.isPreset ? 'border-accent/40 bg-accent/10 scale-[1.02]' : 'border-transparent bg-subtle hover:bg-muted'"
          >
            <div class="w-8 h-8 rounded-full shadow-md" :style="{ backgroundColor: tc.color }"></div>
            <span class="text-[11px]" :class="currentThemeColor === tc.id && settings.isPreset ? 'text-content font-medium' : 'text-content-3'">{{ tc.name }}</span>
          </div>
        </div>

        <!-- 自定义皮肤 -->
        <div class="space-y-2">
          <p class="text-xs text-content-3">自定义</p>
          <div class="grid grid-cols-5 gap-2">
            <div
              v-for="s in settings.customSkins"
              :key="s.id"
              @click="settings.setSkin(s.id)"
              class="flex flex-col items-center gap-1.5 p-2 rounded-xl transition-all border-2 cursor-pointer relative group"
              :class="settings.skin === s.id ? 'border-accent/40 bg-accent/10 scale-[1.02]' : 'border-transparent bg-subtle hover:bg-muted'"
            >
              <div class="w-8 h-8 rounded-full shadow-md relative overflow-hidden" :style="{ backgroundColor: s.preview }">
                <div v-if="s.wallpaper && skinWallpaperDataUrls[s.wallpaper]" class="absolute inset-0 bg-cover bg-center opacity-40" :style="{ backgroundImage: `url(${skinWallpaperDataUrls[s.wallpaper]})` }"></div>
              </div>
              <span class="text-[11px] truncate w-full text-center" :class="settings.skin === s.id ? 'text-content font-medium' : 'text-content-3'">{{ s.name }}</span>
              <!-- 编辑按钮 -->
              <button
                @click.stop="openSkinEditor(s.id)"
                class="absolute -top-1 -right-1 w-4 h-4 flex items-center justify-center rounded-full bg-accent/60 text-white opacity-0 group-hover:opacity-100 transition"
                title="编辑"
              >
                <svg class="w-2 h-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
              </button>
              <!-- 删除按钮 -->
              <button
                @click.stop="handleDeleteCustomSkin(s.id)"
                class="absolute -top-1 -left-1 w-4 h-4 flex items-center justify-center rounded-full bg-danger/80 text-white opacity-0 group-hover:opacity-100 transition"
                title="删除"
              >
                <IconX style="font-size: 8px" />
              </button>
            </div>
            <!-- 创建自定义皮肤（永远排最后） -->
            <div
              @click="openSkinEditor()"
              class="flex flex-col items-center justify-center gap-1.5 p-2 rounded-xl transition-all border-2 border-dashed border-line cursor-pointer hover:border-accent/40 hover:bg-accent/5"
            >
              <div class="w-8 h-8 rounded-full flex items-center justify-center bg-subtle">
                <IconPalette class="w-4 h-4 text-content-3" />
              </div>
              <span class="text-[11px] text-content-3">自定义</span>
            </div>
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
              @click="settings.setShortcut(String(id), defaultShortcuts[id].key)"
              class="w-6 h-6 flex items-center justify-center rounded-md text-content-4 hover:text-danger hover:bg-danger/10 transition"
              title="恢复默认"
            >
              <IconX style="font-size: 14px" />
            </button>
            <button
              @click="startRecording(String(id))"
              class="px-3 py-1.5 rounded-lg text-sm transition min-w-[120px] text-center"
              :class="recordingId === String(id) ? 'bg-accent text-white' : 'bg-muted hover:bg-emphasis text-content-2'"
            >
              {{ recordingId === String(id) ? '按下新快捷键...' : formatShortcut(sc.key) }}
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
            <IconFileText v-if="!updater.checking.value" class="w-4 h-4" />
            <IconLoader2 v-else class="w-4 h-4 animate-spin" />
            {{ updater.checking.value ? '检查中...' : '检查更新' }}
          </button>
          <button
            @click="fetchChangelog"
            :disabled="fetchingChangelog"
            class="flex items-center gap-2 px-4 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition"
          >
            <IconFileText class="w-4 h-4" />
            {{ fetchingChangelog ? '获取中...' : '更新日志' }}
          </button>
        </div>
        <p v-if="updater.error.value" class="text-xs text-content-3">{{ updater.error.value }}</p>
      </div>
    </section>
    <Transition name="fade">
      <div v-if="showResetConfirm" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showResetConfirm = false">
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
      <div v-if="showChangelogModal" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showChangelogModal = false">
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

    <!-- 皮肤编辑器弹窗（Teleport 到 body 避免 z-index 问题） -->
    <Teleport to="body">
    <Transition name="fade">
      <div v-if="showSkinEditor" class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm" @click.self="showSkinEditor = false">
        <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[960px] max-h-[90vh] flex flex-col select-auto">
          <!-- 顶栏 -->
          <div class="flex items-center justify-between p-5 border-b border-line-2">
            <div class="flex items-center gap-3">
              <h2 class="text-lg font-semibold">{{ editingSkinId ? '编辑皮肤' : '创建自定义皮肤' }}</h2>
              <input v-model="editorName" class="px-3 py-1.5 bg-subtle border border-line rounded-lg text-sm text-content focus:border-accent focus:outline-none transition w-40" placeholder="皮肤名称" />
            </div>
            <button @click="showSkinEditor = false" class="text-content-3 hover:text-content transition">
              <IconX class="w-5 h-5" />
            </button>
          </div>

          <div class="flex flex-1 overflow-hidden">
            <!-- 左侧：实时预览 -->
            <div class="w-[420px] flex-shrink-0 p-5 border-r border-line-2 flex flex-col gap-4">
              <p class="text-xs text-content-3 font-medium">实时预览</p>
              <!-- 横向桌面比例预览 -->
              <div class="rounded-xl overflow-hidden border border-line relative" style="aspect-ratio: 16/10;" :style="{ backgroundColor: getEditorColor('bg') }">
                <!-- 壁纸层 -->
                <div v-if="editorWallpaper && editorWallpaperDataUrl" class="absolute inset-0 bg-cover bg-center" :style="{ backgroundImage: `url(${editorWallpaperDataUrl})`, filter: `blur(${editorWallpaperBlur}px)`, opacity: editorWallpaperOpacity }"></div>
                <!-- 无壁纸时的提示 -->
                <div v-if="!editorWallpaper" class="absolute inset-0 flex items-center justify-center">
                  <span class="text-[10px] opacity-20" :style="{ color: getEditorColor('content3') }">纯色背景</span>
                </div>
                <!-- 模拟内容 -->
                <div class="relative z-[1] flex flex-col h-full">
                  <!-- 模拟 TitleBar -->
                  <div class="h-5 flex items-center justify-end px-2 flex-shrink-0" :style="{ backgroundColor: `${getEditorColor('surface')}cc` }">
                    <div class="w-1.5 h-1.5 rounded-full bg-red-500"></div>
                    <div class="w-1.5 h-1.5 rounded-full bg-yellow-500 ml-1"></div>
                    <div class="w-1.5 h-1.5 rounded-full bg-green-500 ml-1"></div>
                  </div>
                  <div class="flex flex-1 min-h-0">
                    <!-- 模拟 Sidebar (w-56 比例) -->
                    <div class="w-[30%] flex-shrink-0 flex flex-col p-1.5 gap-0.5" :style="{ backgroundColor: `${getEditorColor('surface')}cc`, borderRight: `1px solid ${getEditorColor('line')}` }">
                      <div class="flex items-center gap-1 px-1.5 py-1 rounded" :style="{ backgroundColor: getEditorColor('muted') }">
                        <div class="w-1.5 h-1.5 rounded-sm" :style="{ backgroundColor: getEditorColor('accent') }"></div>
                        <div class="h-1 rounded-full w-6" :style="{ backgroundColor: getEditorColor('content') }"></div>
                      </div>
                      <div class="flex items-center gap-1 px-1.5 py-1 rounded">
                        <div class="w-1.5 h-1.5 rounded-sm" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                        <div class="h-1 rounded-full w-5" :style="{ backgroundColor: getEditorColor('content2') }"></div>
                      </div>
                      <div class="flex items-center gap-1 px-1.5 py-1 rounded">
                        <div class="w-1.5 h-1.5 rounded-sm" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                        <div class="h-1 rounded-full w-4" :style="{ backgroundColor: getEditorColor('content2') }"></div>
                      </div>
                      <div class="mt-1 pt-1" :style="{ borderTop: `1px solid ${getEditorColor('line2')}` }">
                        <div class="h-0.5 rounded-full w-4 mb-0.5" :style="{ backgroundColor: getEditorColor('content4') }"></div>
                        <div class="flex items-center gap-1 px-1.5 py-1 rounded">
                          <div class="w-1.5 h-1.5 rounded-sm" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                          <div class="h-1 rounded-full w-7" :style="{ backgroundColor: getEditorColor('content2') }"></div>
                        </div>
                        <div class="flex items-center gap-1 px-1.5 py-1 rounded">
                          <div class="w-1.5 h-1.5 rounded-sm" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                          <div class="h-1 rounded-full w-6" :style="{ backgroundColor: getEditorColor('content2') }"></div>
                        </div>
                      </div>
                      <!-- 底部设置+头像 -->
                      <div class="mt-auto flex items-center gap-1 px-1.5 py-1">
                        <div class="w-3 h-3 rounded-full" :style="{ backgroundColor: getEditorColor('subtle') }"></div>
                        <div class="h-1 rounded-full w-5" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                      </div>
                    </div>
                    <!-- 模拟主内容 -->
                    <div class="flex-1 p-2 flex flex-col gap-1.5 overflow-hidden" :style="editorWallpaper ? { backgroundColor: `${getEditorColor('bg')}cc` } : {}">
                      <div class="h-2 rounded-full w-14" :style="{ backgroundColor: getEditorColor('content') }"></div>
                      <div class="h-1 rounded-full w-20" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                      <!-- 模拟歌曲行 -->
                      <div class="mt-1 flex items-center gap-1 px-1 py-0.5 rounded" :style="{ backgroundColor: `${getEditorColor('surface')}99` }">
                        <div class="w-2 text-right flex-shrink-0"><div class="h-0.5 rounded-full w-1.5 ml-auto" :style="{ backgroundColor: getEditorColor('content4') }"></div></div>
                        <div class="w-4 h-4 rounded flex-shrink-0" :style="{ backgroundColor: getEditorColor('subtle') }"></div>
                        <div class="flex-1 flex flex-col gap-0.5 min-w-0">
                          <div class="h-0.5 rounded-full w-12" :style="{ backgroundColor: getEditorColor('content') }"></div>
                          <div class="h-0.5 rounded-full w-8" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                        </div>
                      </div>
                      <div class="flex items-center gap-1 px-1 py-0.5 rounded">
                        <div class="w-2 text-right flex-shrink-0"><div class="h-0.5 rounded-full w-1.5 ml-auto" :style="{ backgroundColor: getEditorColor('content4') }"></div></div>
                        <div class="w-4 h-4 rounded flex-shrink-0" :style="{ backgroundColor: getEditorColor('subtle') }"></div>
                        <div class="flex-1 flex flex-col gap-0.5 min-w-0">
                          <div class="h-0.5 rounded-full w-10" :style="{ backgroundColor: getEditorColor('content2') }"></div>
                          <div class="h-0.5 rounded-full w-14" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                        </div>
                      </div>
                      <!-- 选中行（均衡器动画） -->
                      <div class="flex items-center gap-1 px-1 py-0.5 rounded" :style="{ backgroundColor: getEditorColor('accentDim') }">
                        <div class="w-2 flex items-center justify-end flex-shrink-0 gap-[1px]">
                          <div class="w-[1px] rounded-full" :style="{ backgroundColor: getEditorColor('accentText'), height: '3px' }"></div>
                          <div class="w-[1px] rounded-full" :style="{ backgroundColor: getEditorColor('accentText'), height: '5px' }"></div>
                          <div class="w-[1px] rounded-full" :style="{ backgroundColor: getEditorColor('accentText'), height: '2px' }"></div>
                        </div>
                        <div class="w-4 h-4 rounded flex-shrink-0" :style="{ backgroundColor: getEditorColor('subtle') }"></div>
                        <div class="flex-1 flex flex-col gap-0.5 min-w-0">
                          <div class="h-0.5 rounded-full w-11" :style="{ backgroundColor: getEditorColor('accentText') }"></div>
                          <div class="h-0.5 rounded-full w-8" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                        </div>
                      </div>
                      <!-- 模拟按钮 -->
                      <div class="flex gap-1 mt-0.5">
                        <div class="px-2 py-0.5 rounded text-[6px] font-medium text-white" :style="{ backgroundColor: getEditorColor('accent') }">播放全部</div>
                        <div class="px-2 py-0.5 rounded text-[6px]" :style="{ backgroundColor: getEditorColor('muted'), color: getEditorColor('content2') }">收藏</div>
                      </div>
                    </div>
                  </div>
                  <!-- 模拟 PlayerBar -->
                  <div class="flex-shrink-0 flex flex-col" :style="{ backgroundColor: `${getEditorColor('surface')}f2` }">
                    <!-- 进度条 -->
                    <div class="h-0.5 w-full" :style="{ backgroundColor: getEditorColor('muted') }">
                      <div class="h-full w-1/3" :style="{ backgroundColor: getEditorColor('accent') }"></div>
                    </div>
                    <div class="flex items-center px-2 h-6 gap-1.5">
                      <!-- 封面+歌名 -->
                      <div class="flex items-center gap-1 w-[30%] min-w-0">
                        <div class="w-4 h-4 rounded flex-shrink-0" :style="{ backgroundColor: getEditorColor('subtle') }"></div>
                        <div class="flex-1 flex flex-col gap-0.5 min-w-0">
                          <div class="h-0.5 rounded-full w-10" :style="{ backgroundColor: getEditorColor('content') }"></div>
                          <div class="h-0.5 rounded-full w-6" :style="{ backgroundColor: getEditorColor('content3') }"></div>
                        </div>
                        <svg class="w-1.5 h-1.5 flex-shrink-0" :style="{ color: getEditorColor('content3') }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
                      </div>
                      <!-- 播放控制 -->
                      <div class="flex-1 flex items-center justify-center gap-2">
                        <svg class="w-2 h-2" :style="{ color: getEditorColor('content2') }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="19 20 9 12 19 4" fill="currentColor"/><line x1="5" y1="4" x2="5" y2="20"/></svg>
                        <div class="w-4 h-4 rounded-full flex items-center justify-center" :style="{ backgroundColor: getEditorColor('muted'), border: `1px solid ${getEditorColor('emphasis')}` }">
                          <svg class="w-2 h-2" :style="{ color: getEditorColor('content') }" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21"/></svg>
                        </div>
                        <svg class="w-2 h-2" :style="{ color: getEditorColor('content2') }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="5 4 15 12 5 20" fill="currentColor"/><line x1="19" y1="4" x2="19" y2="20"/></svg>
                      </div>
                      <!-- 右侧 -->
                      <div class="w-[30%] flex items-center justify-end gap-1">
                        <svg class="w-1.5 h-1.5" :style="{ color: getEditorColor('content3') }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19" fill="currentColor"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
                        <div class="w-4 h-0.5 rounded-full" :style="{ backgroundColor: getEditorColor('muted') }">
                          <div class="h-full w-2/3 rounded-full" :style="{ backgroundColor: getEditorColor('accent') }"></div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <!-- 基础风格快选 -->
              <div>
                <p class="text-xs text-content-3 mb-2">基于预设风格</p>
                <div class="flex gap-1.5 flex-wrap">
                  <div
                    v-for="s in presetSkins"
                    :key="s.id"
                    @click="editorBaseSkin = s.id; onBaseSkinChange()"
                    class="w-6 h-6 rounded-full cursor-pointer border-2 transition-all"
                    :class="editorBaseSkin === s.id ? 'border-white scale-125' : 'border-transparent hover:scale-110'"
                    :style="{ backgroundColor: s.preview }"
                    :title="s.name"
                  ></div>
                </div>
              </div>
            </div>

            <!-- 右侧：控制面板 -->
            <div class="flex-1 overflow-y-auto p-5 space-y-5">
              <!-- 背景与壁纸 -->
              <div class="space-y-3">
                <div class="flex items-center gap-2">
                  <div class="w-1 h-4 rounded-full" :style="{ backgroundColor: getEditorColor('accent') }"></div>
                  <p class="text-sm font-medium">背景与壁纸</p>
                </div>
                <div class="pl-3 space-y-3">
                  <div class="flex items-center gap-3">
                    <label class="color-swatch"><input type="color" :value="toHex(getEditorColor('bg'))" @input="setEditorColor('bg', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm">背景色</p>
                      <p class="text-[11px] text-content-3">整个页面的底色，壁纸会覆盖在上面</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button @click="pickEditorWallpaper" class="flex items-center gap-2 px-3 py-2 bg-subtle hover:bg-muted rounded-lg text-sm transition">
                      <IconImage class="w-4 h-4" />
                      {{ editorWallpaper ? '更换图片' : '选择壁纸图片' }}
                    </button>
                    <button v-if="editorWallpaper" @click="editorWallpaper = ''" class="px-3 py-2 bg-subtle hover:bg-danger/10 rounded-lg text-sm text-content-2 hover:text-danger transition">移除</button>
                  </div>
                  <template v-if="editorWallpaper">
                    <div>
                      <div class="flex justify-between mb-1">
                        <span class="text-xs text-content-3">模糊</span>
                        <span class="text-xs text-content-4">{{ editorWallpaperBlur }}px</span>
                      </div>
                      <input type="range" min="0" max="30" step="1" v-model.number="editorWallpaperBlur" class="w-full h-1.5 bg-muted rounded-full appearance-none cursor-pointer accent-accent" />
                    </div>
                    <div>
                      <div class="flex justify-between mb-1">
                        <span class="text-xs text-content-3">透明度</span>
                        <span class="text-xs text-content-4">{{ Math.round(editorWallpaperOpacity * 100) }}%</span>
                      </div>
                      <input type="range" min="0" max="100" step="5" :value="Math.round(editorWallpaperOpacity * 100)" @input="editorWallpaperOpacity = Number(($event.target as HTMLInputElement).value) / 100" class="w-full h-1.5 bg-muted rounded-full appearance-none cursor-pointer accent-accent" />
                    </div>
                  </template>
                </div>
              </div>

              <!-- 主题色 -->
              <div class="space-y-3">
                <div class="flex items-center gap-2">
                  <div class="w-1 h-4 rounded-full" :style="{ backgroundColor: getEditorColor('accent') }"></div>
                  <p class="text-sm font-medium">主题色</p>
                </div>
                <p class="text-[11px] text-content-3 pl-3">按钮、链接、高亮、播放图标等使用这个颜色</p>
                <div class="pl-3 space-y-2">
                  <div class="flex items-center gap-3">
                    <label class="color-swatch"><input type="color" :value="toHex(getEditorColor('accent'))" @input="setEditorColor('accent', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm">主题色</p>
                      <p class="text-[11px] text-content-3">按钮、进度条、选中状态</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-3">
                    <label class="color-swatch color-swatch-sm"><input type="color" :value="toHex(getEditorColor('accentDim'))" @input="setEditorColor('accentDim', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm text-content-2">主题色淡</p>
                      <p class="text-[11px] text-content-3">选中项的背景、淡色高亮</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 文字颜色 -->
              <div class="space-y-3">
                <div class="flex items-center gap-2">
                  <div class="w-1 h-4 rounded-full" :style="{ backgroundColor: getEditorColor('content') }"></div>
                  <p class="text-sm font-medium">文字</p>
                </div>
                <div class="pl-3 space-y-2">
                  <div class="flex items-center gap-3">
                    <label class="color-swatch"><input type="color" :value="toHex(getEditorColor('content'))" @input="setEditorColor('content', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm">主要文字</p>
                      <p class="text-[11px] text-content-3">标题、歌曲名等最重要的文字</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-3">
                    <label class="color-swatch color-swatch-sm"><input type="color" :value="toHex(getEditorColor('content2'))" @input="setEditorColor('content2', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm text-content-2">次要文字</p>
                      <p class="text-[11px] text-content-3">歌手名、专辑名</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-3">
                    <label class="color-swatch color-swatch-sm"><input type="color" :value="toHex(getEditorColor('content3'))" @input="setEditorColor('content3', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm text-content-2">辅助文字</p>
                      <p class="text-[11px] text-content-3">描述、时间、播放量等</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 表面与卡片 -->
              <div class="space-y-3">
                <div class="flex items-center gap-2">
                  <div class="w-1 h-4 rounded-full" :style="{ backgroundColor: getEditorColor('surface') }"></div>
                  <p class="text-sm font-medium">表面与卡片</p>
                </div>
                <p class="text-[11px] text-content-3 pl-3">侧栏、底栏、弹窗、歌曲卡片的背景色</p>
                <div class="pl-3 space-y-2">
                  <div class="flex items-center gap-3">
                    <label class="color-swatch"><input type="color" :value="toHex(getEditorColor('surface'))" @input="setEditorColor('surface', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm">卡片背景</p>
                      <p class="text-[11px] text-content-3">弹窗、侧栏、底栏的主色</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-3">
                    <label class="color-swatch color-swatch-sm"><input type="color" :value="toHex(getEditorColor('line'))" @input="setEditorColor('line', ($event.target as HTMLInputElement).value)" /></label>
                    <div>
                      <p class="text-sm text-content-2">分割线</p>
                      <p class="text-[11px] text-content-3">卡片边框、区域之间的分隔</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 更多细节（折叠） -->
              <div>
                <button @click="showAdvancedEditor = !showAdvancedEditor" class="flex items-center gap-1.5 text-xs text-content-3 hover:text-content-2 transition">
                  <svg class="w-3 h-3 transition-transform" :class="showAdvancedEditor ? 'rotate-90' : ''" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="9 18 15 12 9 6"/></svg>
                  更多细节调整
                </button>
                <div v-if="showAdvancedEditor" class="mt-3 space-y-4 pl-1">
                  <div>
                    <p class="text-[11px] text-content-3 mb-1.5">悬停与交互</p>
                    <div class="space-y-1.5">
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('subtle'))" @input="setEditorColor('subtle', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">微弱背景</span>
                      </div>
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('muted'))" @input="setEditorColor('muted', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">悬停背景</span>
                      </div>
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('emphasis'))" @input="setEditorColor('emphasis', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">强调背景</span>
                      </div>
                    </div>
                  </div>
                  <div>
                    <p class="text-[11px] text-content-3 mb-1.5">主题色变体</p>
                    <div class="space-y-1.5">
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('accentHover'))" @input="setEditorColor('accentHover', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">按钮悬停</span>
                      </div>
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('accentText'))" @input="setEditorColor('accentText', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">主题文字</span>
                      </div>
                    </div>
                  </div>
                  <div>
                    <p class="text-[11px] text-content-3 mb-1.5">功能色</p>
                    <div class="space-y-1.5">
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('danger'))" @input="setEditorColor('danger', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">危险/错误</span>
                      </div>
                      <div class="flex items-center gap-3">
                        <label class="color-swatch color-swatch-xs"><input type="color" :value="toHex(getEditorColor('warning'))" @input="setEditorColor('warning', ($event.target as HTMLInputElement).value)" /></label>
                        <span class="text-xs text-content-2">警告</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 底栏 -->
          <div class="p-4 border-t border-line flex gap-3">
            <button @click="showSkinEditor = false" class="flex-1 py-2.5 rounded-lg bg-muted hover:bg-emphasis text-sm text-content-2 transition">取消</button>
            <button @click="handleSaveSkin" :disabled="!editorName.trim()" class="flex-1 py-2.5 rounded-lg bg-accent hover:bg-accent-hover text-white text-sm font-medium transition disabled:opacity-50">{{ editingSkinId ? '保存修改' : '创建皮肤' }}</button>
          </div>
        </div>
      </div>
    </Transition>
    </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, reactive, watch } from 'vue';
import { useSettingsStore, qualityLabels, closeActionLabels, defaultShortcuts, type CloseAction } from '../stores/settings';
import { presetSkins, getPresetSkin, type SkinColors } from '../skins';
import { toHex } from '../utils/color';
import { useToast } from '../composables/useToast';
import { useUpdater } from '../composables/useUpdater';
import { DeviceApi, DownloadApi, AppApi } from '../api';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { open } from '@tauri-apps/plugin-dialog';
import CustomSelect from '../components/CustomSelect.vue';
import PageHeader from '../components/PageHeader.vue';
import IconX from '~icons/lucide/x';
import IconFileText from '~icons/lucide/file-text';
import IconLoader2 from '~icons/lucide/loader-2';
import IconPalette from '~icons/lucide/palette';
import IconSun from '~icons/lucide/sun';
import IconMoon from '~icons/lucide/moon';
import IconImage from '~icons/lucide/image';

const settings = useSettingsStore();
const { showToast } = useToast();
const updater = useUpdater();

// 主题色选项（7色，不分深浅）
const themeColorOptions = [
  { id: 'blue', name: '蓝', color: '#3b82f6' },
  { id: 'green', name: '翠', color: '#22c55e' },
  { id: 'rose', name: '红', color: '#f43f5e' },
  { id: 'violet', name: '紫', color: '#8b5cf6' },
  { id: 'orange', name: '橙', color: '#f97316' },
  { id: 'cyan', name: '青', color: '#06b6d4' },
  { id: 'pink', name: '粉', color: '#ec4899' },
];

// 从当前 skin id 解析出 appearance 和 themeColor
const currentAppearance = computed(() => {
  if (settings.skin.startsWith('light')) return 'light';
  return 'dark';
});

const currentThemeColor = computed(() => {
  const id = settings.skin;
  if (id.startsWith('dark-')) return id.slice(5);
  if (id.startsWith('light-')) return id.slice(6);
  return 'blue'; // 自定义皮肤默认蓝
});

function toSkinId(color: string, appearance: 'dark' | 'light'): string {
  return `${appearance}-${color}`;
}

// 壁纸路径转可访问 URL（通过 Rust 命令读取本地图片转 base64 data URL）
const wallpaperCache = new Map<string, string>();
const MAX_WALLPAPER_CACHE = 10;
async function wallpaperSrc(path: string): Promise<string> {
  if (!path) return '';
  if (wallpaperCache.has(path)) return wallpaperCache.get(path)!;
  try {
    const dataUrl = await AppApi.readImageAsDataUrl(path);
    if (wallpaperCache.size >= MAX_WALLPAPER_CACHE) {
      const firstKey = wallpaperCache.keys().next().value;
      if (firstKey !== undefined) wallpaperCache.delete(firstKey);
    }
    wallpaperCache.set(path, dataUrl);
    return dataUrl;
  } catch (e) {
    console.error('加载壁纸预览失败:', e);
    return '';
  }
}
// 用于模板中同步绑定壁纸预览的响应式数据
const editorWallpaperDataUrl = ref('');
const skinWallpaperDataUrls = ref<Record<string, string>>({});

async function loadEditorWallpaper() {
  if (!editorWallpaper.value) {
    editorWallpaperDataUrl.value = '';
    return;
  }
  editorWallpaperDataUrl.value = await wallpaperSrc(editorWallpaper.value);
}

async function loadSkinWallpaperPreviews() {
  for (const s of settings.customSkins) {
    if (s.wallpaper && !skinWallpaperDataUrls.value[s.wallpaper]) {
      const url = await wallpaperSrc(s.wallpaper);
      if (url) skinWallpaperDataUrls.value[s.wallpaper] = url;
    }
  }
}

// 皮肤编辑器
const showSkinEditor = ref(false);
const showAdvancedEditor = ref(false);
const editorName = ref('');
const editorBaseSkin = ref('dark-blue');
const editorColors = reactive<Partial<SkinColors>>({});
const editorWallpaper = ref('');
const editorWallpaperBlur = ref(10);
const editorWallpaperOpacity = ref(0.3);
/** 正在编辑的已有皮肤 id，为空则表示创建新皮肤 */
const editingSkinId = ref<string | null>(null);

function openSkinEditor(skinId?: string) {
  if (skinId) {
    // 编辑已有自定义皮肤
    const existing = settings.customSkins.find(s => s.id === skinId);
    if (!existing) return;
    editingSkinId.value = skinId;
    editorName.value = existing.name;
    editorBaseSkin.value = 'dark-blue';
    // 将已有颜色完整填入 editorColors
    Object.keys(editorColors).forEach(k => delete editorColors[k as keyof SkinColors]);
    for (const [key, value] of Object.entries(existing.colors)) {
      (editorColors as any)[key] = value;
    }
    editorWallpaper.value = existing.wallpaper || '';
    editorWallpaperBlur.value = existing.wallpaperBlur ?? 10;
    editorWallpaperOpacity.value = existing.wallpaperOpacity ?? 0.3;
  } else {
    // 创建新皮肤：基于当前皮肤或默认
    editingSkinId.value = null;
    editorName.value = '';
    const baseSkinId = settings.isPreset ? settings.skin : 'dark-blue';
    editorBaseSkin.value = baseSkinId;
    // 将基础皮肤颜色完整填入 editorColors
    Object.keys(editorColors).forEach(k => delete editorColors[k as keyof SkinColors]);
    const base = getPresetSkin(baseSkinId);
    if (base) {
      for (const [key, value] of Object.entries(base.colors)) {
        (editorColors as any)[key] = value;
      }
    }
    editorWallpaper.value = '';
    editorWallpaperBlur.value = 10;
    editorWallpaperOpacity.value = 0.3;
  }
  showSkinEditor.value = true;
  loadEditorWallpaper();
  loadSkinWallpaperPreviews();
}

function onBaseSkinChange() {
  // 切换基础风格时，将该风格的完整颜色填入 editorColors
  Object.keys(editorColors).forEach(k => delete editorColors[k as keyof SkinColors]);
  const base = getPresetSkin(editorBaseSkin.value);
  if (base) {
    for (const [key, value] of Object.entries(base.colors)) {
      (editorColors as any)[key] = value;
    }
  }
}

function getEditorColor(key: keyof SkinColors): string {
  return editorColors[key] || '#000000';
}

function setEditorColor(key: keyof SkinColors, value: string) {
  editorColors[key] = value;
}

function handleSaveSkin() {
  if (!editorName.value.trim()) return;
  // 确保颜色完整：缺失字段从基础皮肤补齐
  const base = getPresetSkin(editorBaseSkin.value);
  const baseColors = base ? base.colors : getPresetSkin('dark-blue')!.colors;
  const colors = { ...baseColors } as SkinColors;
  for (const key of Object.keys(editorColors) as (keyof SkinColors)[]) {
    if (editorColors[key]) {
      colors[key] = editorColors[key]!;
    }
  }

  if (editingSkinId.value) {
    settings.updateCustomSkin(editingSkinId.value, {
      name: editorName.value.trim(),
      preview: colors.accent,
      colors,
      wallpaper: editorWallpaper.value,
      wallpaperBlur: editorWallpaperBlur.value,
      wallpaperOpacity: editorWallpaperOpacity.value,
    });
    showSkinEditor.value = false;
    showToast('皮肤已更新', 'success');
  } else {
    const id = `custom-${Date.now()}`;
    settings.addCustomSkin({
      id,
      name: editorName.value.trim(),
      preview: colors.accent,
      colors,
      wallpaper: editorWallpaper.value,
      wallpaperBlur: editorWallpaperBlur.value,
      wallpaperOpacity: editorWallpaperOpacity.value,
    });
    showSkinEditor.value = false;
    showToast('自定义皮肤已创建', 'success');
  }
}

function handleDeleteCustomSkin(id: string) {
  settings.removeCustomSkin(id);
  showToast('已删除自定义皮肤', 'success');
}

async function pickEditorWallpaper() {
  const selected = await open({
    multiple: false,
    title: '选择壁纸图片',
    filters: [{
      name: '图片',
      extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'],
    }],
  });
  if (selected) {
    editorWallpaper.value = selected;
    wallpaperCache.delete(selected);
    loadEditorWallpaper();
  }
}


// 监听编辑器壁纸变化
watch(editorWallpaper, () => {
  loadEditorWallpaper();
});

// 监听自定义皮肤列表变化，加载壁纸预览
watch(() => settings.customSkins, () => {
  loadSkinWallpaperPreviews();
}, { deep: true });

const devices = ref<string[]>([]);
const deviceOptions = computed(() => {
  const options: Record<string, string> = { '': '跟随系统默认' };
  for (const name of devices.value) {
    options[name] = name;
  }
  return options;
});

const selectedDevice = computed({
  get: () => settings.outputDevice || '',
  set: (val: string) => {
    const device = val === '' ? null : val;
    settings.setOutputDevice(device);
    DeviceApi.setOutputDevice(device).then(() => {
      showToast(device ? `已切换到: ${device}` : '已切换到系统默认', 'success');
    }).catch((e) => {
      console.error('切换设备失败: ', e);
      showToast('切换设备失败', 'error');
    });
  }
});

async function loadDevices() {
  try {
    devices.value = await DeviceApi.getOutputDevices();
  } catch (e) {
    console.error('获取设备失败: ', e);
  }
}

const appVersion = ref('');
const defaultDownloadPath = ref('');
onMounted(async () => {
  appVersion.value = await getVersion();
  try {
    defaultDownloadPath.value = await DownloadApi.getDefaultDownloadPath();
  } catch { /* 忽略 */ }
  loadDevices();
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
    .replace(/Key([A-Z])/g, '$1')
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
    const hasModifier = parts.includes('Control') || parts.includes('Alt') || parts.includes('Shift');
    if (!hasModifier) {
      showToast('快捷键必须包含 Ctrl、Alt 或 Shift', 'error');
      recordingId.value = null;
      return;
    }
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

<style scoped>
/* 颜色选择器：让 input[type=color] 填满外层 label，消除内部小方块 */
.color-swatch {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  border: 1px solid var(--color-line);
  overflow: hidden;
  cursor: pointer;
  flex-shrink: 0;
  width: 36px;
  height: 36px;
}
.color-swatch-sm {
  width: 28px;
  height: 28px;
  border-radius: 6px;
}
.color-swatch-xs {
  width: 24px;
  height: 24px;
  border-radius: 5px;
}
.color-swatch input[type="color"] {
  -webkit-appearance: none;
  appearance: none;
  border: none;
  padding: 0;
  margin: 0;
  width: calc(100% + 8px);
  height: calc(100% + 8px);
  cursor: pointer;
  background: transparent;
}
.color-swatch input[type="color"]::-webkit-color-swatch-wrapper {
  padding: 0;
}
.color-swatch input[type="color"]::-webkit-color-swatch {
  border: none;
  border-radius: 0;
}
.color-swatch input[type="color"]::-moz-color-swatch {
  border: none;
}
</style>
