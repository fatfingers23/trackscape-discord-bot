<script setup lang="ts">

import DataTable from "@/components/DataTable.vue";
import type {ClanDetail} from "@/services/TrackscapeApiTypes";
import type {PropType} from "vue";

const props = defineProps({
  clanDetail: {
    type: Object as PropType<ClanDetail>,
    required: true
  }
})

const columns = [
  {
    name: 'Members',
    key: 'player_name'
  },
  {
    name: 'View WOM',
    key: 'player_name'
  }
];


</script>

<template>
  <div v-if="props.clanDetail !== undefined"
       class="p-5 shadow-xl bg-base-100 " >
    <div class="overflow-x-auto">
      <DataTable :columns="columns"
                 :data="props.clanDetail.members"
                 search-field="player_name"

      >
        <template #row-item="{item, column}" >
          <template v-if="column.name == 'View WOM'">
            <a class="link text-sm md:text-base"
               :href="`https://www.wiseoldman.net/players/${item.player_name}`"> View WOM</a></template>
          <template v-else>
            <span class="text-sma md:text-base">
              {{item[column.key]}}
            </span>
          </template>
        </template>
      </DataTable>
    </div>
  </div>
</template>

<style scoped>

</style>
