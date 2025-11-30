// src/router/index.js
import { createRouter, createWebHashHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue'; // Example view
import AboutView from '../views/AboutView.vue'; // Example view

const routes = [
  {
    path: '/',
    name: 'home',
    component: HomeView,
  },
  {
    path: '/about',
    name: 'about',
    component: AboutView,
  },
  // Add other routes here
];

const router = createRouter({
  history: createWebHashHistory(), // Use hash history for Tauri
  routes,
});

export default router;
