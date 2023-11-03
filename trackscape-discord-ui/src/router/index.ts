import { createRouter, createWebHistory } from 'vue-router'
import BotLandingPage from '../views/BotLandingPage.vue'
import ClanList from "@/views/ClanList.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'bot-landing-page',
      component: BotLandingPage
    },
    {
      path: '/clans',
      name: 'clan-list',
      component: ClanList
    }
  ]
})

export default router
