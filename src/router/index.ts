import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  { path: '/', name: 'home', component: () => import('@/views/Home.vue') },
  { path: '/discover', name: 'discover', component: () => import('@/views/Discover.vue') },
  { path: '/search', name: 'search', component: () => import('@/views/Discover.vue') },
  { path: '/favorites', name: 'favorites', component: () => import('@/views/FavoriteSongs.vue') },
  { path: '/recent', name: 'recent', component: () => import('@/views/RecentPlays.vue') },
  { path: '/daily', name: 'daily', component: () => import('@/views/DailySongs.vue') },
  { path: '/local-music', name: 'local-music', component: () => import('@/views/LocalMusic.vue') },
  { path: '/downloaded-music', name: 'downloaded-music', component: () => import('@/views/DownloadedMusic.vue') },
  { path: '/cloud-music', name: 'cloud-music', component: () => import('@/views/CloudMusic.vue') },
  { path: '/login', name: 'login', component: () => import('@/views/Login.vue'), meta: { guest: true } },
  { path: '/playlist/:id', name: 'playlist', component: () => import('@/views/PlaylistDetail.vue') },
  { path: '/artist/:id', name: 'artist', component: () => import('@/views/ArtistDetail.vue') },
  { path: '/album/:id', name: 'album', component: () => import('@/views/AlbumDetail.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/views/Settings.vue') },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to) => {
  if (to.meta.guest) {
    const raw = localStorage.getItem('user_profile');
    if (raw) {
      try {
        const data = JSON.parse(raw);
        if (data?.userId) return { name: 'home' };
      } catch { /* 忽略 */ }
    }
  }
});

export default router;
