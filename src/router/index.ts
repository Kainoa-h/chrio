import { createRouter, createWebHashHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import AboutView from '../views/AboutView.vue';
import ClientSessionsView from '../views/ClientSessionsView.vue';
import AddSessionView from '../views/AddSessionView.vue';
import CompareSessionsView from '../views/CompareSessionsView.vue';

const routes = [
  {
    path: '/',
    name: 'home',
    component: HomeView,
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
  {
    path: '/client/:id/sessions/:sessionId/edit',
    name: 'edit-session',
    component: AddSessionView,
  },
  {
    path: '/client/:clientId/compare',
    name: 'compare-sessions',
    component: CompareSessionsView,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
