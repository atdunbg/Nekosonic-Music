<template>
  <div class="bg-subtle rounded-xl p-3" ref="scrollContainer">
    <div v-if="loading" class="py-8 text-center text-content-2 text-sm">加载中...</div>

    <div v-else-if="comments.length === 0" class="py-8 text-center">
      <svg class="mx-auto mb-2 text-content-3" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
      <p class="text-content-3 text-sm">暂无评论</p>
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="comment in comments"
        :key="comment.commentId"
        class="p-3 rounded-xl bg-surface/50"
      >
        <div class="flex items-center gap-3">
          <img :src="comment.user.avatarUrl" class="w-8 h-8 rounded-full object-cover flex-shrink-0" />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-content truncate">{{ comment.user.nickname }}</p>
            <p class="text-xs text-content-3">{{ new Date(comment.time).toLocaleDateString('zh-CN') }}</p>
          </div>
        </div>
        <p class="mt-2 text-sm text-content-2 leading-relaxed">{{ comment.content }}</p>
        <div class="mt-2 flex justify-end">
          <button
            @click="likeComment(comment.commentId)"
            class="flex items-center gap-1 text-content-3 hover:text-danger transition text-xs"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z"/></svg>
            <span>{{ comment.likedCount }}</span>
          </button>
        </div>
      </div>
      <div ref="sentinel" class="h-1"></div>
    </div>

    <div v-if="loadingMore" class="py-4 text-center text-content-3 text-sm">加载中...</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  type: number
  id: number
}>()

const comments = ref<any[]>([])
const loading = ref(false)
const loadingMore = ref(false)
const hasMore = ref(true)
const pageNo = ref(1)
const pageSize = 20
const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

async function fetchComments(reset = false) {
  if (reset) {
    pageNo.value = 1
    comments.value = []
    hasMore.value = true
  }

  if (!hasMore.value) return

  if (reset) {
    loading.value = true
  } else {
    loadingMore.value = true
  }

  try {
    const jsonStr: string = await invoke('comment_hot', {
      query: {
        type: props.type,
        id: props.id,
        limit: pageSize,
        offset: (pageNo.value - 1) * pageSize
      }
    })
    const data = JSON.parse(jsonStr)
    const list = data.hotComments || []
    if (reset) {
      comments.value = list
    } else {
      comments.value.push(...list)
    }
    hasMore.value = list.length >= pageSize
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
    loadingMore.value = false
  }
}

function loadMore() {
  if (loadingMore.value || !hasMore.value) return
  pageNo.value++
  fetchComments()
}

async function likeComment(cid: number) {
  try {
    await invoke('comment_like', {
      query: {
        t: 1,
        type: props.type,
        id: props.id,
        cid
      }
    })
    const target = comments.value.find(c => c.commentId === cid)
    if (target) {
      target.likedCount++
    }
  } catch (e) {
    console.error(e)
  }
}

function setupObserver() {
  if (observer) observer.disconnect()
  observer = new IntersectionObserver((entries) => {
    if (entries[0]?.isIntersecting) {
      loadMore()
    }
  }, { rootMargin: '200px' })
  nextTick(() => {
    if (sentinel.value) {
      observer!.observe(sentinel.value)
    }
  })
}

onMounted(() => {
  fetchComments(true).then(() => setupObserver())
})

watch(() => props.id, () => {
  fetchComments(true).then(() => setupObserver())
})

onBeforeUnmount(() => {
  if (observer) observer.disconnect()
})
</script>
