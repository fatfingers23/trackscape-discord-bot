
import type {RouteRecordRaw} from "vue-router";

export default [
    {
        path: '/clans',
        name: 'clan-list',
        component: () => import('@/views/clans/ClanList.vue'),
    },
    {
        path: '/clans/:clanId/detail',
        name: 'clan-detail',
        component: () => import('@/views/clans/ClanDetail.vue')
    }
] as RouteRecordRaw[];
