<script setup lang="ts">

</script>

<template>
  <div class="navbar bg-base-100">
    <div class="navbar-start">
      <div class="dropdown">
        <label tabindex="0"
               class="btn btn-ghost lg:hidden">
          <svg xmlns="http://www.w3.org/2000/svg"
               class="h-5 w-5"
               fill="none"
               viewBox="0 0 24 24"
               stroke="currentColor">
            <path stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 6h16M4 12h8m-8 6h16"/>
          </svg>
        </label>

      </div>
      <a href="/"
         tabindex="0"
         class="btn btn-ghost btn-circle avatar">
        <div class="w-10 rounded-full">
          <img src="@/assets/img/Trackscape_Logo_icon.png"/>
        </div>
      </a>
      <a href="/"
         class="btn btn-ghost normal-case text-xl md:visible invisible">Trackscape</a>

      <!--      <a href="{{route('clanlanding-page', $clan->name)}}"-->
      <!--         class="btn btn-ghost normal-case text-xl">{{$clan->name}}</a>-->

    </div>
    <div class="navbar-end md:flex">
      <ul class="menu menu-horizontal p-0 hidden lg:flex">
        <li>
          <router-link :to="{name: 'clan-list'}"
                       class="btn btn-ghost">Clans</router-link>
        </li>

        <!--        <li tabindex="0">-->
        <!--          <a>-->
        <!--            Leaderboards-->
        <!--            <svg class="fill-current"-->
        <!--                 xmlns="http://www.w3.org/2000/svg"-->
        <!--                 width="20"-->
        <!--                 height="20"-->
        <!--                 viewBox="0 0 24 24">-->
        <!--              <path d="M7.41,8.58L12,13.17L16.59,8.58L18,10L12,16L6,10L7.41,8.58Z"/>-->
        <!--            </svg>-->
        <!--          </a>-->
        <!--          <ul class="p-2 bg-base-100">-->
        <!--            <li><a class=""-->
        <!--                   href="">Collection-->
        <!--              Log Leaderboard</a>-->
        <!--            </li>-->
        <!--            <li><a class=""-->
        <!--                   href="">PB Leaderboard</a>-->
        <!--            </li>-->
        <!--          </ul>-->
        <!--        </li>-->



      <!--        <li><a class=""-->
      <!--               href="">FAQ</a></li>-->

      <!--        <li><a class=""-->
      <!--               href="">Clan Lookup</a></li>-->
      </ul>

    </div>
  </div>

</template>

<style scoped>

</style>
