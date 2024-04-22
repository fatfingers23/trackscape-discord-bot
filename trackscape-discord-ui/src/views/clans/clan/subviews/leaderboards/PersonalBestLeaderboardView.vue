<script setup lang="ts">
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import type { ClanDetail, ClanMateCollectionLogTotalsView, PbActivity } from '@/services/TrackscapeApiTypes';
import type {PropType} from "vue";
import {useRoute} from "vue-router";
import {ref} from "vue";
import DataTable from "@/components/General/DataTable.vue";
import SkeletonTable from "@/components/General/SkeletonTable.vue";
import { usePbStore } from '@/stores/PbStore';
import ClanMateWithRank from '@/components/clan/ClanMateWithRank.vue';
import { it } from 'vitest';

const client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);
const store = usePbStore();

const props = defineProps({
  clanDetail: {
    type: Object as PropType<ClanDetail>,
    required: true
  }
});


// const getPbs = (activityId: string) => client.getCollectionLogLeaderboard(id).then((leaderboard) => {
//   let rank = 1;
//   leaderboard.map((item) => {
//     item.rank = rank;
//     rank++;
//   });
//   // collectionLogLeaderboard.value = leaderboard
// });


let clan = ref<ClanDetail>();
// let collectionLogLeaderboard = ref<ClanMateCollectionLogTotalsView[]>();

if (props.clanDetail) {
  clan.value = props.clanDetail;
  // callEndpoint(props.clanDetail.id);
} else {
  const route = useRoute();
  const clanId = route.params.clanId as string;
  // callEndpoint(clanId);
}

const columns = [
  {
    name: 'Rank',
    key:'rank'
  },
  {
    name: 'Member',
    key: 'clan_mate.player_name'
  },
  {
    name: 'Personal Best',
    key: 'time_in_seconds'
  }
];
</script>
<template>
  <div v-if="props.clanDetail !== undefined"
       class="p-5 shadow-xl bg-base-100 " >
    <div class="">
      <TransitionGroup name="slide-fade">

        <SkeletonTable v-if="store.records.length === 0"
                       :search-field="true"
                       :columns="3"/>


        <DataTable
          v-else
          :title="`${store.$state.selectedActivityName} Leaderboard`"
          :columns="columns"
          :data="store.getRecords"
          search-field="clan_mate.player_name"
        >
          <template #row-item="{item, column}" >
            <div class="text-sma md:text-base">
              <span v-if="column.key == 'time_in_seconds'">
                {{item[column.key].toLocaleString()}}
              </span>
              <span v-else-if="column.key === 'clan_mate.player_name'">
                <ClanMateWithRank :rank="item.clan_mate.rank"
                                  :name="item.clan_mate.player_name" />
              </span >
              <span v-else>
                {{item[column.key]}}
              </span>
            </div>
          </template>
        </DataTable>
      </TransitionGroup>
    </div>
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
