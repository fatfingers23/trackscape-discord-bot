<script setup lang="ts">
import type { Broadcast } from '@/services/TrackscapeApiTypes'
import { type PropType, ref } from 'vue'

const props = defineProps({
  broadcast: {
    type: Object as PropType<Broadcast>,
    required: true
  }
})

</script>

<template>
  <div class="p-2 shadow-xl bg-base-100">
    <div class="flex items-center gap-x-7">
      <div class="flex items-center">
        <img v-if="props.broadcast.broadcast.icon_url !== null"
             class="w-10 h-10 rounded-full object-contain mr-1"
             :src="props.broadcast.broadcast.icon_url"
             onerror="this.src = '/src/assets/img/Trackscape_Logo_icon.png'"
             alt=""
        />

        <div>
          <p class="text-md font-bold">{{ props.broadcast.broadcast.title }}</p>
          <p class="text-sm text-gray-500">{{ props.broadcast.created_at }}</p>
        </div>
      </div>
    </div>
    <div class="mt-1">
      <p>{{ props.broadcast.broadcast.message }}</p>
    </div>
  </div>
</template>

<style scoped>

</style>
