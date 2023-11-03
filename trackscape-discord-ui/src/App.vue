<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'

import NavMenu from "@/components/nav-menu.vue";
</script>

<template>
  <nav-menu/>
  <main class="bg-base-200">
    <RouterView />
    <footer class="
        footer
        p-10
        bg-neutral
        text-neutral-content">
      <div>
        <div class="w-10 rounded-full">
          <img class="rounded-full"
               src="@/assets/img/Trackscape_Logo_icon.png"/>
        </div>
        <p>Trackscape.<br>OSRS tooling for clans!</p>
      </div>

    </footer>
  </main>

</template>

<style scoped>

</style>
