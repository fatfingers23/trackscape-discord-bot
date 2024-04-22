<script setup lang="ts">
import type {PropType} from "vue";


const props = defineProps({
  rank: {
    type: String,
    required: false,
    default: 'Not Ranked',
  },
  name: {
    type: String,
    required: true,
  }
});


let showRank: boolean;

if(props.rank === null){
  showRank = false;
}else{
  showRank = props.rank !== 'Not Ranked';
}

</script>

<template>
  <span
    class="flex text-sm md:text-base">
    <img
      v-if="showRank"
      class="w-6 h-6 rounded-full"
      :alt="`${props?.rank} icon`"
      :src="`https://oldschool.runescape.wiki/images/Clan_icon_-_${props?.rank == 'Deputy Owner'  ? 'Deputy_owner' : props?.rank.replace(' ', '_')}.png`" />
    {{ props.name }}
  </span>
</template>
