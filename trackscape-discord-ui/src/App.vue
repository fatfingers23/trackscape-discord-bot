<script setup lang="ts">
import { RouterView } from 'vue-router';

import NavMenu from "@/components/nav-menu.vue";
import Drawer from "@/components/NavDrawer.vue";
</script>

<template>

  <Drawer>
    <template #content>
      <nav-menu/>
      <main class="bg-base-200">
        <div class="container mx-auto md:p-8 p-3 min-h-screen">
          <RouterView />
        </div>
        <footer class="
        footer
        p-10
        bg-neutral
        text-neutral-content">
          <div>
            <div class="w-10 rounded-full">
              <img class="rounded-full"
                   src="@/assets/img/Trackscape_Logo_icon.png"
                   alt="Trackscape logo"/>
            </div>
            <p>Trackscape<br>OSRS tooling for clans!</p>
            <a class="link"
               href="https://github.com/fatfingers23/trackscape-discord-bot">Just show me the code</a>

            <RouterLink class="link"
                        to="/TermsOfService">Terms Of Service</RouterLink>
            <RouterLink class="link"
                        to="/PrivacyPolicy">Privacy Policy</RouterLink>
          </div>

        </footer>
      </main>
    </template>
  </Drawer>



</template>

<style scoped>

</style>
