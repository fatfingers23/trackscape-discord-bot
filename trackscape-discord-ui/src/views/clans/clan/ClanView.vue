<script setup lang="ts">

import {useRoute} from "vue-router";
import PageTitle from "@/components/PageTitle.vue";
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import {ref} from "vue";
import type {  ClanDetail } from '@/services/TrackscapeApiTypes'
import DiscordWidget from "@/components/DiscordWidget.vue";
import BroadcastList from '@/components/BroadcastList.vue'
import { useHead } from '@unhead/vue'

let client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);

const route = useRoute()
const clanId = route.params.clanId as string;

let clanDetail = ref<ClanDetail>()

client.getClanDetail(clanId).then((clan) => {
  //put clan list in alphabetical order
  clan.members =  clan.members.sort(function (a, b) {
    if (a.player_name < b.player_name) {
      return -1;
    }
    if (a.player_name > b.player_name) {
      return 1;
    }
    return 0;
  });
  clanDetail.value = clan;
  useHead({
    title: `${clan.name} - TrackScape`,
  })
});




const tabMenus = [
  {
    name: 'Members',
    routeName: 'members',
    active: true
  },
  {
    name: 'Recent Broadcasts',
    routeName: 'broadcasts',
    active: true
  },
  {
    name: 'Collection Logs',
    routeName: 'collection-log',
    active: false
  },
]

</script>

<template>

  <PageTitle
    :title="clanDetail?.name ?? ''">
    <div class="mt-2 flex items-center gap-x-7">
      <div class="flex items-center ">
        <svg
          class="mr-2 h-5 w-10 text-gray-200"
          version="1.1"
          xmlns="http://www.w3.org/2000/svg"
          xmlns:xlink="http://www.w3.org/1999/xlink"
          width="29.707"
          height="16.5723">
          <g>
            <rect height="16.5723"
                  opacity="0"
                  width="29.707"
                  x="0"
                  y="0"/>
            <path d="M29.3457 14.1016C29.3457 14.9902 28.7891 15.4297 27.6855 15.4297L22.1951 15.4297C22.4362 15.0649 22.5488 14.6285 22.5488 14.1504C22.5488 14.1472 22.5488 14.144 22.5481 14.1406L27.9199 14.1406C27.998 14.1406 28.0469 14.1016 28.0469 14.0137C28.0469 12.207 25.9277 10.4785 23.4863 10.4785C22.6896 10.4785 21.9265 10.6641 21.2624 10.9796C20.9899 10.6418 20.6718 10.3191 20.3105 10.0217C21.2324 9.49869 22.3284 9.18945 23.4863 9.18945C26.6504 9.18945 29.3457 11.4844 29.3457 14.1016ZM26.4062 5.14648C26.4062 6.9043 25.0879 8.34961 23.4863 8.34961C21.9043 8.34961 20.5762 6.91406 20.5762 5.16602C20.5762 3.45703 21.8945 2.05078 23.4863 2.05078C25.1172 2.05078 26.4062 3.42773 26.4062 5.14648ZM21.875 5.16602C21.875 6.20117 22.627 7.05078 23.4863 7.05078C24.3652 7.05078 25.1172 6.20117 25.1172 5.14648C25.1172 4.13086 24.3945 3.34961 23.4863 3.34961C22.6074 3.34961 21.875 4.15039 21.875 5.16602Z"
                  fill="#ffffff"
                  fill-opacity="0.85"/>
            <path d="M9.03063 10.0206C8.66886 10.3179 8.35039 10.6405 8.07746 10.9782C7.41259 10.6635 6.64835 10.4785 5.84961 10.4785C3.41797 10.4785 1.29883 12.207 1.29883 14.0137C1.29883 14.1016 1.34766 14.1406 1.42578 14.1406L6.7878 14.1406C6.78711 14.144 6.78711 14.1472 6.78711 14.1504C6.78711 14.6285 6.90089 15.0649 7.14355 15.4297L1.66016 15.4297C0.556641 15.4297 0 14.9902 0 14.1016C0 11.4844 2.69531 9.18945 5.84961 9.18945C7.0102 9.18945 8.10773 9.49823 9.03063 10.0206ZM8.76953 5.14648C8.76953 6.9043 7.45117 8.34961 5.85938 8.34961C4.26758 8.34961 2.93945 6.91406 2.93945 5.16602C2.93945 3.45703 4.25781 2.05078 5.85938 2.05078C7.48047 2.05078 8.76953 3.42773 8.76953 5.14648ZM4.23828 5.16602C4.23828 6.20117 4.99023 7.05078 5.85938 7.05078C6.72852 7.05078 7.48047 6.20117 7.48047 5.14648C7.48047 4.13086 6.75781 3.34961 5.85938 3.34961C4.9707 3.34961 4.23828 4.15039 4.23828 5.16602Z"
                  fill="#ffffff"
                  fill-opacity="0.85"/>
            <path d="M14.6777 8.30078C16.5137 8.30078 18.0371 6.65039 18.0371 4.66797C18.0371 2.68555 16.5332 1.13281 14.6777 1.13281C12.8223 1.13281 11.3086 2.71484 11.3086 4.6875C11.3086 6.66016 12.832 8.30078 14.6777 8.30078ZM14.6777 6.99219C13.6328 6.99219 12.7246 5.9668 12.7246 4.6875C12.7246 3.4082 13.6035 2.45117 14.6777 2.45117C15.752 2.45117 16.6211 3.38867 16.6211 4.66797C16.6211 5.94727 15.7227 6.99219 14.6777 6.99219ZM9.95117 15.4297L19.3945 15.4297C20.7617 15.4297 21.416 15.0293 21.416 14.1504C21.416 12.1387 18.8672 9.19922 14.668 9.19922C10.4785 9.19922 7.92969 12.1387 7.92969 14.1504C7.92969 15.0293 8.58398 15.4297 9.95117 15.4297ZM9.50195 14.1113C9.39453 14.1113 9.33594 14.082 9.33594 13.9746C9.33594 12.832 11.2207 10.5176 14.668 10.5176C18.125 10.5176 20 12.832 20 13.9746C20 14.082 19.9512 14.1113 19.8438 14.1113Z"
                  fill="#ffffff"
                  fill-opacity="0.85"/>
          </g>

        </svg>
        <span class="text-xs text-gray-200">{{clanDetail?.registered_members}} (Only those so far recorded)</span>
      </div>
    </div>
  </PageTitle>

  <div class="container bg-base-200">
    <DiscordWidget
      class="mx-3"
      v-if="clanDetail !== undefined"
      :discord_id="clanDetail?.discord_guild_id" />
    <div
      class="pt-3 pb-3 tabs tabs-bordered min-w-full">
      <router-link v-for="(tabMenu,index) in tabMenus"
                   :key="index"
                   :to="{name: tabMenu.routeName, params: {clanId: clanId}}"
                   :class="['tab', {'tab-active': route.name === tabMenu.routeName}]">{{tabMenu.name}}</router-link>
    </div>

    <router-view v-slot="{ Component}" >
      <component :is="Component"
                 :clanDetail="clanDetail" />
    </router-view>
    <BroadcastList v-if="route.name !== 'broadcasts'"
                   class="pt-2"
                   :clan-id="clanId" />
  </div>


</template>

<style scoped>

</style>
