<template>
  <div class="min-h-screen flex items-center justify-center bg-base text-content">
    <div class="bg-subtle backdrop-blur-md border border-line p-8 rounded-2xl w-full max-w-sm text-center">
      <h1 class="text-xl font-bold mb-4">扫码登录</h1>
      <p class="text-sm text-content-2 mb-6">请使用网易云音乐 App 扫描二维码</p>

      <div v-if="qrimg" class="bg-white p-3 rounded-xl inline-block mb-4">
        <img :src="qrimg" alt="二维码" class="w-48 h-48" />
      </div>
      <div v-else class="w-48 h-48 bg-subtle rounded-xl flex items-center justify-center mx-auto mb-4">
        <span v-if="qrLoading" class="text-content-2">加载中...</span>
        <span v-else-if="qrError" class="text-danger text-sm">{{ qrError }}</span>
      </div>

      <p class="text-sm" :class="statusColor">{{ statusText }}</p>
      <button @click="refreshQr" class="mt-4 text-xs text-accent-text hover:underline">重新获取二维码</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'vue-router';
import { useUserStore } from '../stores/user';
import QRCode from 'qrcode';

const router = useRouter();
const userStore = useUserStore();
const qrimg = ref('');
const qrLoading = ref(true);
const qrError = ref('');
const statusText = ref('等待扫码...');
const statusColor = ref('text-content-2');
let qrKey = '';
let pollTimer: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  if (userStore.isLoggedIn) {
    router.push('/');
    return;
  }
  await refreshQr();
});

onBeforeUnmount(() => {
    if (pollTimer) clearInterval(pollTimer);
});

async function refreshQr() {
    qrLoading.value = true;
    qrError.value = '';
    if (pollTimer) clearInterval(pollTimer);
    try {
        qrKey = await invoke('get_qr_key');
        if (!qrKey) {
            qrError.value = '未获取到登录密钥';
            qrLoading.value = false;
            return;
        }

        const qrUrl = `https://music.163.com/login?codekey=${qrKey}&type=1`;

        const canvas = document.createElement('canvas');
        await QRCode.toCanvas(canvas, qrUrl, { width: 200, margin: 1 });
        qrimg.value = canvas.toDataURL('image/png');
        qrLoading.value = false;

        startPolling();
    } catch (e: any) {
        qrError.value = '获取二维码失败';
        qrLoading.value = false;
    }
}

function startPolling() {
    pollTimer = setInterval(async () => {
        try {
            const jsonStr: string = await invoke('check_qr_status', { query: { key: qrKey } });
            const data = JSON.parse(jsonStr);
            const code = data.code;
            if (code === 800) {
                statusText.value = '二维码已过期，请刷新';
                statusColor.value = 'text-danger';
                clearInterval(pollTimer!);
            } else if (code === 801) {
                statusText.value = '等待扫码...';
                statusColor.value = 'text-content-2';
            } else if (code === 802) {
                statusText.value = '请在手机上确认登录';
                statusColor.value = 'text-warning';
            } else if (code === 803) {
                clearInterval(pollTimer!);
                statusText.value = '登录成功！';
                statusColor.value = 'text-accent-text';
                await fetchUserProfile();
                setTimeout(() => router.push('/'), 500);
            }
        } catch (e) {
            console.error('轮询失败', e);
        }
    }, 3000);
}

async function fetchUserProfile() {
    try {
        const profileJson: string = await invoke('get_login_status');
        const profile = JSON.parse(profileJson);
        if (profile.profile) {
            userStore.setUser({
                userId: profile.profile.userId,
                nickname: profile.profile.nickname,
                avatarUrl: profile.profile.avatarUrl,
            });
        }
    } catch (e) {
        console.error('获取用户信息失败', e);
    }
}
</script>
