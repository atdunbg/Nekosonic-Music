import { createRouter, createWebHistory } from 'vue-router';
import Home from '@/views/Home.vue';
import Discover from '@/views/Discover.vue';
import PlaylistDetail from '@/views/PlaylistDetail.vue';
import Login from '@/views/Login.vue';
import FavoriteSongs from '@/views/FavoriteSongs.vue';
import RecentPlays from '@/views/RecentPlays.vue';
import DailySongs from '@/views/DailySongs.vue';


const routes = [
  { path: '/', name: 'home', component: Home },
  { path: '/discover', name: 'discover', component: Discover },
  { path: '/search', name: 'search', component: Discover }, // 同样指向Discover，保留兼容
  { path: '/roam', name: 'roam', component: () => import('@/views/Roam.vue') }, // 漫游页面
  { path: '/favorites', name: 'favorites', component: FavoriteSongs },
  { path: '/recent', name: 'recent', component: RecentPlays },
  { path: '/daily', name: 'daily', component: DailySongs }, // 每日推荐
  { path: '/login', name: 'login', component: Login },
  { path: '/playlist/:id', name: 'playlist', component: PlaylistDetail },
];

export default createRouter({
  history: createWebHistory(),
  routes,
});