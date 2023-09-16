import { createRouter, createWebHistory } from 'vue-router'
import BotLandingPage from '../views/BotLandingPage.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'bot-landing-page',
      component: BotLandingPage
    },

  ]
})

export default router
