<script setup lang="ts">

import {RouterView} from "vue-router";
import NavMenu from "@/components/nav-menu.vue";
</script>

<template>
  <div class="drawer">
    <input id="my-drawer"
           type="checkbox"
           class="drawer-toggle" />
    <div class="drawer-content">
      <slot name="content"></slot>
      <label for="my-drawer"
             class="btn btn-primary drawer-button">Open drawer</label>
    </div>
    <div class="drawer-side">
      <label for="my-drawer"
             aria-label="close sidebar"
             class="drawer-overlay"></label>



      <ul class="menu p-4 w-80 min-h-full bg-base-200 text-base-content">
        <!-- Sidebar content here -->
        <li><a>Sidebar Item 1</a></li>
        <li><a>Sidebar Item 2</a></li>

      </ul>
    </div>
  </div>

</template>

<style scoped>

</style>
