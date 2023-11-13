<script setup lang="ts">
import {computed, ref} from "vue";
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import type {Clan} from "@/services/TrackscapeApiTypes";
import PageTitle from "@/components/PageTitle.vue";

let client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);
let clans = ref<Clan[]>([]);
let search = ref<string>("");

client.getClans().then((apiClans: Clan[]) => {
  clans.value = apiClans;

});

let displayedClans = computed(() => {
  return clans.value.filter((clan) => {
    return clan.name.toLowerCase().includes(search.value.toLowerCase());
  });
});

</script>

<template>

  <PageTitle title="Clans">
    <input
      v-model="search"
      type="text"
      placeholder="Type clan name here"
      class="input input-bordered w-full md:max-w-md max-w-full" />
  </PageTitle>

  
  <TransitionGroup name="list"
                   tag="div"
                   class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <div v-for="(clan, index) in displayedClans"
         :key="index">
      <div class="card bg-primary text-primary-content card-compact shadow-sm shadow-accent-content">
        <div class="card-body">
          <h2 class="card-title">{{clan.name}}
          </h2>
          <span class="text-sm">{{clan.registered_members}} clanmates</span>

          <!--            <p>Clan info</p>-->
          <div class="card-actions justify-end">
            <router-link :to="{name: 'clan-detail', params: {clanId: clan.id}}"
                         class="btn ">View</router-link>

          </div>
        </div>
      </div>
      <router-view></router-view>
    </div>
  </TransitionGroup>

</template>

<style scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.5s ease;
}
.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
