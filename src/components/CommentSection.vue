<template>
  <div class="bg-subtle rounded-xl p-3" ref="scrollContainer">
    <div v-if="loading" class="py-8 text-center text-sm" :class="darkMode ? 'text-white/60' : 'text-content-2'">加载中...</div>

    <div v-else-if="comments.length === 0" class="py-8 text-center">
      <IconMessageSquare class="mx-auto mb-2 w-10 h-10" :class="darkMode ? 'text-white/40' : 'text-content-3'" />
      <p class="text-sm" :class="darkMode ? 'text-white/40' : 'text-content-3'">暂无评论</p>
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="comment in comments"
        :key="comment.commentId"
        class="p-3 rounded-xl"
        :class="darkMode ? 'bg-white/8' : 'bg-surface/50'"
      >
        <div class="flex items-center gap-3">
          <img :src="comment.user.avatarUrl" class="w-8 h-8 rounded-full object-cover flex-shrink-0" />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate" :class="darkMode ? 'text-white/90' : 'text-content'">{{ comment.user.nickname }}</p>
            <p class="text-xs" :class="darkMode ? 'text-white/40' : 'text-content-3'">{{ new Date(comment.time).toLocaleDateString('zh-CN') }}</p>
          </div>
        </div>
        <p class="mt-2 text-sm leading-relaxed" :class="darkMode ? 'text-white/70' : 'text-content-2'">{{ comment.content }}</p>
        <div class="mt-2 flex justify-end">
          <button
            @click="likeComment(comment.commentId)"
            class="flex items-center gap-1 transition text-xs"
            :class="comment.liked ? 'text-danger' : (darkMode ? 'text-white/40 hover:text-danger' : 'text-content-3 hover:text-danger')"
          >
            <IconHeart style="font-size: 14px" :class="comment.liked ? '[&>path]:fill-current [&>path]:stroke-0' : ''" />
            <span>{{ comment.likedCount }}</span>
          </button>
        </div>
      </div>
      <div ref="sentinel" class="h-1"></div>
    </div>

    <div v-if="loadingMore" class="py-4 text-center text-sm" :class="darkMode ? 'text-white/40' : 'text-content-3'">加载中...</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { MusicApi } from '../api'
import IconMessageSquare from '~icons/lucide/message-square'
import IconHeart from '~icons/lucide/heart'

const props = defineProps<{
  type: number
  id: number
  darkMode?: boolean
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
    const jsonStr: string = await MusicApi.commentHot({
      type: props.type,
      id: props.id,
      limit: pageSize,
      offset: (pageNo.value - 1) * pageSize
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

const likingSet = ref(new Set<number>())

async function likeComment(cid: number) {
  if (likingSet.value.has(cid)) return
  const target = comments.value.find(c => c.commentId === cid)
  if (!target) return
  const liked = !!target.liked
  likingSet.value.add(cid)
  try {
    await MusicApi.commentLike({
      t: liked ? 0 : 1,
      type: props.type,
      id: props.id,
      cid
    })
    target.liked = !liked
    target.likedCount += liked ? -1 : 1
  } catch (e) {
    console.error(e)
  } finally {
    likingSet.value.delete(cid)
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
