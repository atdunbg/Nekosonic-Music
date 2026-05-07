import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface UserProfile {
  userId: number;
  nickname: string;
  avatarUrl: string;
}

export const useUserStore = defineStore('user', () => {
  const user = ref<UserProfile | null>(
    JSON.parse(localStorage.getItem('user_profile') || 'null')
  );
  const isLoggedIn = ref(!!user.value);

  function setUser(profile: UserProfile) {
    user.value = profile;
    isLoggedIn.value = true;
    localStorage.setItem('user_profile', JSON.stringify(profile));
  }

  async function logout() {
    try { await invoke('logout'); } catch {}
    user.value = null;
    isLoggedIn.value = false;
    localStorage.removeItem('user_profile');
  }

  return { user, isLoggedIn, setUser, logout };
});