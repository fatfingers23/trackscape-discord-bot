import { createRouter, createWebHistory } from 'vue-router'
import BotLandingPage from '../views/BotLandingPage.vue'
import ClanRoutes from './clans';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  scrollBehavior(to, from, savedPosition) {
    // always scroll to top
    return { top: 0 }
  },
  routes: [
    {
      path: '/',
      name: 'bot-landing-page',
      component: BotLandingPage
    },
      ...ClanRoutes
  ]
})

export default router
