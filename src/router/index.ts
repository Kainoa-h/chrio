import { createRouter, createWebHashHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import AboutView from '../views/AboutView.vue';
import ClientSessionsView from '../views/ClientSessionsView.vue';
import AddSessionView from '../views/AddSessionView.vue';

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
  {
    path: '/client/:id/sessions',
    name: 'client-sessions',
    component: ClientSessionsView,
  },
  {
    path: '/client/:id/sessions/new',
    name: 'add-session',
    component: AddSessionView,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
