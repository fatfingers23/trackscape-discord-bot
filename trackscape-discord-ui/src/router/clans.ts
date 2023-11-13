import type {RouteRecordRaw} from "vue-router";

export default [
    {
        path: '/clans',
        name: 'clan-list',
        component: () => import('@/views/clans/ClanListView.vue'),
    },
    {
        path: '/clans/:clanId',
        name: 'clan',
        component: () => import('@/views/clans/clan/ClanView.vue'),
        children: [
            {
                path: "",
                name: "clan-detail",
                component: () => import('@/views/clans/clan/ClanDetailView.vue'),
                children: []
            }
        ]

    },
    // {
    //     path: '/clans/:clanId/detail',
    //     name: 'clan-detail',
    //     component: () => import('@/views/clans/clan/ClanView.vue')
    // }
] as RouteRecordRaw[];
