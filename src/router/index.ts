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
  { path: '/roam', name: 'roam', component: () => import('@/views/Roam.vue') },
  { path: '/favorites', name: 'favorites', component: FavoriteSongs },
  { path: '/recent', name: 'recent', component: RecentPlays },
  { path: '/daily', name: 'daily', component: DailySongs },
  { path: '/local-music', name: 'local-music', component: LocalMusic },
  { path: '/login', name: 'login', component: Login },
  { path: '/playlist/:id', name: 'playlist', component: PlaylistDetail },
  { path: '/artist/:id', name: 'artist', component: () => import('@/views/ArtistDetail.vue') },
  { path: '/album/:id', name: 'album', component: () => import('@/views/AlbumDetail.vue') },
  { path: '/settings', name: 'settings', component: Settings },
];

export default createRouter({
  history: createWebHistory(),
  routes,
});