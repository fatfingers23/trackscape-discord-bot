<script setup lang="ts">
import type {ClanDetail} from "@/services/TrackscapeApiTypes";
import { type PropType } from 'vue';
import {useRoute} from "vue-router";
import {ref} from "vue";
import BroadcastList from '@/components/clan/BroadcastList.vue';


let clanId = ref<string>();
const props = defineProps({
  clanDetail: {
    type: Object as PropType<ClanDetail>,
    required: true
  }
});

if (props.clanDetail) {
  clanId.value= props.clanDetail.id;
} else {
  const route = useRoute();
  clanId.value = route.params.clanId as string;
}


</script>
<template>
  <div v-if="clanId === undefined">
    a
  </div>
  <BroadcastList v-else
                 :limit="99"
                 :showHeader="false"
                 :clan-id="clanId" />

</template>

<style scoped>
.slide-fade-enter-active {
  transition: all 0.1s ease-in;
}

.slide-fade-leave-active {
  transition: all 0.1s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(-20px);
  opacity: 0;
}
</style>
