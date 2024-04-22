<script setup lang="ts">

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

</script>

<template>
  <span
    class="flex text-sm md:text-base">
    <img
      v-if="props?.rank !== 'Not Ranked' && props?.rank !== null"
      class="w-6 h-6 rounded-full"
      :alt="`${props?.rank} icon`"
      :src="`https://oldschool.runescape.wiki/images/Clan_icon_-_${props?.rank == 'Deputy Owner'  ? 'Deputy_owner' : props?.rank.replace(' ', '_')}.png`" />
    {{ props.name }}
  </span>
</template>
