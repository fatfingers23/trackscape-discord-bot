import type { RouteRecordRaw } from 'vue-router'

export default [
  {
    path: '/clans',
    name: 'clan-list',
    component: () => import('@/views/clans/ClanListView.vue')
  },
  {
    path: '/clans/:clanId',
    name: 'clan',
    component: () => import('@/views/clans/clan/ClanView.vue'),
    children: [
      {
        path: '',
        name: 'members',
        component: () => import('@/views/clans/clan/subviews/MembersView.vue')

      },
      {
        path: 'leaderboards',
        name: 'leaderboards',
        component: () => import('@/views/clans/clan/subviews/LeaderboardView.vue'),
        children: [
          {
            path: 'collectionlog',
            name: 'collection-log',
            component: () => import('@/views/clans/clan/subviews/leaderboards/CollectionLogLeaderboardView.vue')
          },
          {
            path: 'pb',
            name: 'personal-best',
            component: () => import('@/views/clans/clan/subviews/leaderboards/PersonalBestLeaderboardView.vue')
          }
        ]
      },
      {
        path: 'broadcasts',
        name: 'broadcasts',
        component: () => import('@/views/clans/clan/subviews/RecentBroadcastsView.vue')
      }
    ]
  }
  // {
  //     path: '/clans/:clanId/detail',
  //     name: 'clan-detail',
  //     component: () => import('@/views/clans/clan/ClanView.vue')
  // }
] as RouteRecordRaw[]
