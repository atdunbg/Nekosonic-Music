import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'

export interface UpdateInfo {
  version: string
  date: string | null
  body: string | null
}

const IGNORED_VERSION_KEY = 'updater_ignored_version'

export function useUpdater() {
  const checking = ref(false)
  const downloading = ref(false)
  const downloadProgress = ref(0)
  const updateAvailable = ref(false)
  const updateInfo = ref<UpdateInfo | null>(null)
  const currentVersion = ref('')
  const error = ref('')

  async function getCurrentVersion() {
    try {
      currentVersion.value = await getVersion()
    } catch {
      currentVersion.value = ''
    }
  }

  function getIgnoredVersion(): string {
    try {
      return localStorage.getItem(IGNORED_VERSION_KEY) || ''
    } catch {
      return ''
    }
  }

  function setIgnoredVersion(version: string) {
    try {
      localStorage.setItem(IGNORED_VERSION_KEY, version)
    } catch {}
  }

  async function checkForUpdate(silent = false): Promise<UpdateInfo | null> {
    if (checking.value) return null
    checking.value = true
    error.value = ''
    updateAvailable.value = false
    updateInfo.value = null

    try {
      await getCurrentVersion()
      const result = await check()

      if (!result) {
        if (!silent) error.value = '当前已是最新版本'
        return null
      }

      const info: UpdateInfo = {
        version: result.version,
        date: result.date ?? null,
        body: result.body ?? null,
      }

      const ignored = getIgnoredVersion()
      if (info.version === ignored) {
        if (!silent) error.value = '当前已是最新版本'
        return null
      }

      updateAvailable.value = true
      updateInfo.value = info
      return info
    } catch (e: any) {
      if (!silent) error.value = `检查更新失败: ${e}`
      return null
    } finally {
      checking.value = false
    }
  }

  async function downloadAndInstall() {
    if (downloading.value) return
    downloading.value = true
    downloadProgress.value = 0
    error.value = ''

    try {
      const result = await check()
      if (!result) {
        error.value = '未找到可用更新'
        return
      }

      let downloaded = 0
      let contentLength = 0
      await result.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength ?? 0
            break
          case 'Progress':
            downloaded += event.data.chunkLength
            if (contentLength > 0) {
              downloadProgress.value = Math.round((downloaded / contentLength) * 100)
            }
            break
          case 'Finished':
            downloadProgress.value = 100
            break
        }
      })

      await relaunch()
    } catch (e: any) {
      error.value = `更新失败: ${e}`
    } finally {
      downloading.value = false
    }
  }

  function ignoreVersion(version: string) {
    setIgnoredVersion(version)
    updateAvailable.value = false
    updateInfo.value = null
  }

  return {
    checking,
    downloading,
    downloadProgress,
    updateAvailable,
    updateInfo,
    currentVersion,
    error,
    checkForUpdate,
    downloadAndInstall,
    ignoreVersion,
    getCurrentVersion,
  }
}
