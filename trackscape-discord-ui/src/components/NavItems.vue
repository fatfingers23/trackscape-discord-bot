<script setup lang="ts">

import ThemePicker from "@/components/ThemePicker.vue";
import {useRoute} from "vue-router";
const route = useRoute();
</script>

<template>
  <li>
    <router-link :to="{name: 'clan-list'}"
                 :class="{'active': route.name === 'clan-list'}">Clans</router-link>
  </li>
  <li>
    <ThemePicker/>
  </li>
</template>

<style scoped>

</style>
