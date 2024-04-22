<script setup lang="ts">
import BroadcastItem from '@/components/clan/BroadcastItem.vue';
import { ref } from 'vue';
import type { Broadcast } from '@/services/TrackscapeApiTypes';
import TrackscapeApiClient from '@/services/TrackscapeApiClient';

let broadcasts = ref<Broadcast[]>();
let loading = ref(true);
let props = defineProps({
  clanId: {
    type: String,
    required: true
  },
  limit: {
    type: Number,
    required: false,
    default: 10
  },
  showHeader: {
    type: Boolean,
    required: false,
    default: true
  }
});


let client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);

client.getBroadcasts(props.clanId, props.limit).then((broadcastsResult) => {
  broadcastsResult.forEach((broadcast) => {
    broadcast.created_at = new Date(broadcast.created_at).toLocaleString();
    broadcast.broadcast.title = broadcast.broadcast.title.replace(/:.*:/, '');
  });
  broadcasts.value = broadcastsResult;
  loading.value = false;
});

</script>

<template>
  <TransitionGroup name="slide-fade">

    <div v-if="loading">
      <div class="flex items-center gap-x-2">
      </div>
      <div class="mt-2">
        <div v-for="index in 10"
             :key="index"
             class="p-2 shadow-xl bg-base-100">
          <div class="p-2 shadow-xl bg-base-100">
            <div class="flex items-center ">
              <div class="flex items-center">
                <div class="skeleton w-10 h-10 rounded-full shrink-0"></div>
                <div class="flex flex-col gap-1">
                  <div class="skeleton h-6 w-28"></div>
                  <div class="skeleton h-4 w-28"></div>
                </div>
              </div>
            </div>
            <div class="mt-1 flex flex-col gap-1">
              <div class="skeleton h-4 w-3/4"></div>
              <div class="skeleton h-4 w-3/4"></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else>
      <div class="flex items-center gap-x-2">
        <div v-if="props.showHeader"
             class="flex items-center">
          <div>
            <p class="text-lg font-bold">Recent Broadcasts</p>
          </div>
        </div>
      </div>
      <div class="mt-2">
        <div v-if="broadcasts?.length === 0">
          <div class="p-2 shadow-xl bg-base-100">
            <p>Your clan has not gotten any broadcasts. Go play the game!</p>
          </div>
        </div>
        <div v-for="broadcast in broadcasts"
             :key="broadcast.id"
             class="p-5 shadow-xl bg-base-100">
          <BroadcastItem :broadcast="broadcast" />
        </div>
      </div>
    </div>
  </TransitionGroup>


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
