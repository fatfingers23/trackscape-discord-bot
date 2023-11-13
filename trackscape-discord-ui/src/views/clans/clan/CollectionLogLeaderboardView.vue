<script setup lang="ts">
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import type {ClanDetail, ClanMateCollectionLogTotalsView} from "@/services/TrackscapeApiTypes";
import type {PropType} from "vue";
import {useRoute} from "vue-router";
import {ref} from "vue";
import DataTable from "@/components/DataTable.vue";

const client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);

const props = defineProps({
  clanDetail: {
    type: Object as PropType<ClanDetail>,
    required: true
  }
})

const callEndpoint = (id: string) => client.getCollectionLogLeaderboard(id).then((leaderboard) => {
  collectionLogLeaderboard.value = leaderboard;
});


let clan = ref<ClanDetail>();
let collectionLogLeaderboard = ref<ClanMateCollectionLogTotalsView[]>();

if (props.clanDetail) {
  clan.value = props.clanDetail;
  callEndpoint(props.clanDetail.id);
} else {
  const route = useRoute();
  const clanId = route.params.clanId as string;
  callEndpoint(clanId);
}

const columns = [
  {
    name: 'Member',
    key: 'player_name'
  },
  {
    name: 'Total',
    key: 'total'
  }
];
</script>
<template>
  <div v-if="props.clanDetail !== undefined"
       class="p-5 shadow-xl bg-base-100 " >
    <div class="overflow-x-auto">
      <DataTable
        v-if="collectionLogLeaderboard !== undefined"
        :columns="columns"
        :data="collectionLogLeaderboard"

      >

      </DataTable>
    </div>
  </div>
</template>

<style scoped>

</style>
