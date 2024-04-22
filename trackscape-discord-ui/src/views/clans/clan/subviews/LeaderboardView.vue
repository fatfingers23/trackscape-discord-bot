<script setup lang="ts">
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import type {ClanDetail, ClanMateCollectionLogTotalsView} from "@/services/TrackscapeApiTypes";
import type {PropType} from "vue";
import {useRoute} from "vue-router";
import {ref} from "vue";
import DataTable from "@/components/General/DataTable.vue";
import SkeletonTable from "@/components/General/SkeletonTable.vue";
import PersonalBestActivitySelect from '@/components/clan/PersonalBestActivitySelect.vue'



const props = defineProps({
  clanDetail: {
    type: Object as PropType<ClanDetail>,
    required: true
  }
})


let clanId = ref<string>();
const route = useRoute()

if (props.clanDetail) {
  clanId.value = props.clanDetail.id;
} else {
  const route = useRoute();
  clanId.value = route.params.clanId as string;
}

const leaderBoardMenus = [
  {
    name: 'Collection Logs',
    routeName: 'collection-log',
    active: false
  },
  {
    name: 'Personal Best Times',
    routeName: 'personal-best',
    active: false
  },
];
</script>
<template>
  <div class="pt-1 justify-between items-center flex md:flex-row flex-col">
    <div
      role="tablist"
      class="tabs tabs-boxed md:w-1/2 w-full">
      <router-link v-for="(menu, index) in leaderBoardMenus"
                   :key="index"
                   :to="{name: menu.routeName, params: {clanId: clanId}}"
                   :class="['tab', {'tab-active': menu.routeName === route.name}]"
                   role="tab"
      >{{ menu.name }}</router-link>
    </div>
    <PersonalBestActivitySelect v-if="route.name === 'personal-best'"/>
  </div>
  <div class="container bg-base-200">

    <router-view v-slot="{ Component}" >
      <component :is="Component"
                 :clanDetail="clanDetail" />
    </router-view>
  </div>
</template>

<style scoped>
.slide-fade-enter-active {
  transition: all 0.1s ease-in;
}

.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(-20px);
  opacity: 0;
}
</style>
