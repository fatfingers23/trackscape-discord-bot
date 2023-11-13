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
                name: "members",
                component: () => import('@/views/clans/clan/MembersView.vue'),

            },
            {
                path: "collectionlog",
                name: "collection-log",
                component: () => import('@/views/clans/clan/CollectionLogLeaderboardView.vue'),
            }
        ]

    },
    // {
    //     path: '/clans/:clanId/detail',
    //     name: 'clan-detail',
    //     component: () => import('@/views/clans/clan/ClanView.vue')
    // }
] as RouteRecordRaw[];
