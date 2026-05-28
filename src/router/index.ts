import { createRouter, createWebHistory } from 'vue-router';
import Home from '@/views/Home.vue';
import Discover from '@/views/Discover.vue';
import PlaylistDetail from '@/views/PlaylistDetail.vue';
import Login from '@/views/Login.vue';
import FavoriteSongs from '@/views/FavoriteSongs.vue';
import RecentPlays from '@/views/RecentPlays.vue';
import DailySongs from '@/views/DailySongs.vue';
import LocalMusic from '@/views/LocalMusic.vue';
import Settings from '@/views/Settings.vue';

const routes = [
  { path: '/', name: 'home', component: Home },
  { path: '/discover', name: 'discover', component: Discover },
  { path: '/search', name: 'search', component: Discover },
  { path: '/favorites', name: 'favorites', component: FavoriteSongs },
  { path: '/recent', name: 'recent', component: RecentPlays },
  { path: '/daily', name: 'daily', component: DailySongs },
  { path: '/local-music', name: 'local-music', component: LocalMusic },
  { path: '/login', name: 'login', component: Login, meta: { guest: true } },
  { path: '/playlist/:id', name: 'playlist', component: PlaylistDetail },
  { path: '/artist/:id', name: 'artist', component: () => import('@/views/ArtistDetail.vue') },
  { path: '/album/:id', name: 'album', component: () => import('@/views/AlbumDetail.vue') },
  { path: '/settings', name: 'settings', component: Settings },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to) => {
  if (to.meta.guest) {
    const raw = localStorage.getItem('user');
    if (raw) {
      try {
        const data = JSON.parse(raw);
        if (data?.userId) return { name: 'home' };
      } catch { /* 忽略 */ }
    }
  }
});

export default router;
