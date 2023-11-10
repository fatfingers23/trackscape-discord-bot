<script setup lang="ts">
import {ref} from "vue";
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import type {Clan} from "@/services/TrackscapeApiTypes";

let client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);
let clans = ref<Clan[]>([]);
client.getClans().then((apiClans: Clan[]) => {
  clans.value = apiClans;
});


</script>

<template>
  <div class="container mx-auto p-10 min-h-screen flex flex-col ">
    <div class="overflow-x-auto">
      <table class="table">
        <!-- head -->
        <thead>
          <tr>
            <th>Name</th>
          </tr>
        </thead>
        <tbody>

          <tr v-for="(clan, index) in clans"
              :key="index">
            <th>{{clan.name}}</th>

          </tr>

        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>

</style>
