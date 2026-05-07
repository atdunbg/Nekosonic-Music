<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-950 text-white">
    <div class="bg-white/5 backdrop-blur-md border border-white/10 p-8 rounded-2xl w-full max-w-sm text-center">
      <h1 class="text-xl font-bold mb-4">扫码登录</h1>
      <p class="text-sm text-gray-400 mb-6">请使用网易云音乐 App 扫描二维码</p>
      
      <!-- 二维码展示区 -->
      <div v-if="qrimg" class="bg-white p-3 rounded-xl inline-block mb-4">
        <img :src="qrimg" alt="二维码" class="w-48 h-48" />
      </div>
      <div v-else class="w-48 h-48 bg-white/5 rounded-xl flex items-center justify-center mx-auto mb-4">
        <span v-if="qrLoading" class="text-gray-400">加载中...</span>
        <span v-else-if="qrError" class="text-red-400 text-sm">{{ qrError }}</span>
      </div>
      
      <!-- 状态提示 -->
      <p class="text-sm" :class="statusColor">{{ statusText }}</p>
      <button @click="refreshQr" class="mt-4 text-xs text-green-400 hover:underline">重新获取二维码</button>
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
const statusColor = ref('text-gray-400');
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
        // 1. 获取 unikey
        qrKey = await invoke('get_qr_key');
        if (!qrKey) {
            qrError.value = '未获取到登录密钥';
            qrLoading.value = false;
            return;
        }

        // 2. 拼接网易云标准扫码链接（无需 create_qr）
        const qrUrl = `https://music.163.com/login?codekey=${qrKey}&type=1`;

        // 3. 用 qrcode 生成二维码图片
        const canvas = document.createElement('canvas');
        await QRCode.toCanvas(canvas, qrUrl, { width: 200, margin: 1 });
        qrimg.value = canvas.toDataURL('image/png');
        qrLoading.value = false;

        // 4. 开始轮询状态
        startPolling();
    } catch (e: any) {
        qrError.value = '获取二维码失败';
        qrLoading.value = false;
    }
}

// 新增函数：用 Canvas 生成二维码并赋值给 qrimg
// async function drawQrCode(url: string) {
//     try {
//         // 等待 DOM 准备好 canvas 元素
//         const canvas = document.createElement('canvas');
//         await QRCode.toCanvas(canvas, url, { width: 201, margin: 1 });
//         // 转为 data URL 赋值给响应式的图片地址
//         qrimg.value = canvas.toDataURL('image/png');
//     } catch (e) {
//         console.error('生成二维码失败', e);
//         qrError.value = '生成二维码失败';
//     }
// }

function startPolling() {
    pollTimer = setInterval(async () => {
        try {
            const jsonStr: string = await invoke('check_qr_status', { query: { key: qrKey } });
            const data = JSON.parse(jsonStr);
            const code = data.code;
            if (code === 800) {
                statusText.value = '二维码已过期，请刷新';
                statusColor.value = 'text-red-400';
                clearInterval(pollTimer!);
            } else if (code === 801) {
                statusText.value = '等待扫码...';
                statusColor.value = 'text-gray-400';
            } else if (code === 802) {
                statusText.value = '请在手机上确认登录';
                statusColor.value = 'text-yellow-400';
            } else if (code === 803) {
                // 登录成功
                clearInterval(pollTimer!);
                statusText.value = '登录成功！';
                statusColor.value = 'text-green-400';
                // 存储 cookie 到 NcmApi（后台线程中自动保留，后续请求都带登录态）
                // 获取用户信息（简化，可从 /login/status 获取）
                // 这里需要额外调用获取用户详情的 API，但因为 NcmApi 已有 cookie，可以直接在后台线程中添加
                // 暂时用简易方式：调用 /user/account 获取用户简档
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
        // 添加一个快速获取用户信息的命令（可复用之前的 login 命令中获取 profile 的逻辑）
        // 这里简化，由于后台 NcmApi 已有 cookie，我们可以直接用 reqwest 调 /user/account
        // 但最好添加一个新命令，这里直接调用现有的 login 逻辑不适用，因此我们在 Rust 侧添加一个 get_login_status 命令
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