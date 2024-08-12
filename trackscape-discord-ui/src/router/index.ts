import { createRouter, createWebHistory } from 'vue-router';
import BotLandingPage from '../views/BotLandingPage.vue';
import ClanRoutes from './clans';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  scrollBehavior() {
    // always scroll to top
    return { top: 0 };
  },
  routes: [
    {
      path: '/',
      name: 'bot-landing-page',
      component: BotLandingPage
    },
    {
      path: '/TermsOfService',
      name:'TermsOfService',
      component:  () => import('@/views/borningStuff/TermsOfService.vue'),
    },
    {
      path: '/PrivacyPolicy',
      name:'PrivacyPolicy',
      component:  () => import('@/views/borningStuff/PrivacyPolicy.vue'),
    },
      ...ClanRoutes
  ]
});

export default router;
