<script setup lang="ts">
import { ref } from 'vue'
import type { PbActivity } from '@/services/TrackscapeApiTypes'
import TrackscapeApiClient from '@/services/TrackscapeApiClient'
import { usePbStore } from '@/stores/PbStore'

const client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);
const store = usePbStore();

let activities = ref<PbActivity[]>();

client.getTrackScapePbActivities().then((activitiesFromApi) => {
  console.log(activitiesFromApi);
  // activities.value = activitiesFromApi.map((activity) => {
  //   return {
  //     label: activity.activity_name,
  //     value: activity._id
  //   }
  // }) as SelectItem[];
  activities.value = activitiesFromApi;
  store.setSelectedActivity(activities.value[0]);
});
</script>

<template>
  <div class="md:pt-0 pt-2">
    <select v-model="store.$state.selectedActivity"

            class="select w-full">
      <option disabled
              selected>Which leaderboard?</option>
      <option v-for="activity in activities"
              :key="activity._id"
              :value="activity._id">{{activity.activity_name}}</option>
    </select>

    <!--    <TextSearchSelect v-if="activities !== undefined && selectedActivity !== undefined"-->
    <!--                      :options="activities"-->
    <!--                      :model-value="selectedActivity"-->

    <!--    />-->

  </div>
</template>

<style scoped>

</style>
