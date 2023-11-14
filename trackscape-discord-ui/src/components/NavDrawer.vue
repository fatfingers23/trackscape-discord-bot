<script setup lang="ts">
import NavItems from "@/components/NavItems.vue";
import {ref} from "vue";

let open = ref(false);
</script>

<template>
  <div class="drawer">
    <input
      v-model="open"
      id="mobile-drawer"
      type="checkbox"
      class="drawer-toggle" />
    <div class="drawer-content">
      <slot name="content"></slot>

    </div>
    <div class="drawer-side">
      <label for="mobile-drawer"
             aria-label="close sidebar"
             class="drawer-overlay"></label>

      <ul class="menu p-4 w-80 min-h-full bg-base-200 text-base-content">
        <!-- Sidebar content here -->
        <li>

          <router-link
            @click="open = false"
            to="/"
            tabindex="0"
            class="btn-circle avatar">

            <div class="w-10 rounded-full">
              <img src="@/assets/img/Trackscape_Logo_icon.png"
                   alt="Small circle trackscape icon"/>
            </div>
            Trackscape
          </router-link>

        </li>
        <li class="divider-accent"></li>
        <nav-items @click="open = false"/>
      </ul>
    </div>
  </div>

</template>

<style scoped>

</style>
