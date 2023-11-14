<script setup lang="ts">
import NavItems from "@/components/NavItems.vue";
</script>

<template>
  <div class="navbar bg-base-100">
    <div class="navbar-start">
      <div >
        <label for="mobile-drawer"
               aria-label="open sidebar"
               class="btn btn-square btn-ghost md:hidden">
          <svg xmlns="http://www.w3.org/2000/svg"
               fill="none"
               viewBox="0 0 24 24"
               class="inline-block w-6 h-6 stroke-current"><path stroke-linecap="round"
                                                                 stroke-linejoin="round"
                                                                 stroke-width="2"
                                                                 d="M4 6h16M4 12h16M4 18h16"></path></svg>
        </label>

      </div>
      <router-link to="/"
                   tabindex="0"
                   class="btn btn-ghost btn-circle avatar md:visible invisible">
        <div class="w-10 rounded-full">
          <img
            alt="Small circle trackscape icon"
            src="@/assets/img/Trackscape_Logo_icon.png"/>
        </div>
      </router-link>
      <router-link to="/"
                   class="btn btn-ghost normal-case text-xl md:visible invisible md:visible invisible">Trackscape</router-link>



    </div>
    <div class="navbar-end md:flex">
      <ul class="menu menu-horizontal p-0 hidden lg:flex">
        <nav-items/>
      </ul>
    </div>
  </div>

</template>

<style scoped>

</style>
