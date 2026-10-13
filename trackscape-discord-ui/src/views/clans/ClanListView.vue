<script setup lang="ts">
import {computed, onBeforeUnmount, ref, watch} from "vue";
import TrackscapeApiClient from "@/services/TrackscapeApiClient";
import type {Clan} from "@/services/TrackscapeApiTypes";
import PageTitle from "@/components/PageTitle.vue";
import { useHead } from '@unhead/vue';

const PAGE_SIZE = 50;
const SEARCH_DEBOUNCE_MS = 300;

const client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);
const clans = ref<Clan[]>([]);
const search = ref<string>("");
const total = ref<number>(0);
const loading = ref<boolean>(true);
const loadingMore = ref<boolean>(false);
const error = ref<boolean>(false);

let searchDebounce: ReturnType<typeof setTimeout> | null = null;

const hasMore = computed(() => clans.value.length < total.value);

async function loadClans(offset: number, searchTerm: string, more: boolean) {
  if (more) {
    loadingMore.value = true;
  } else {
    loading.value = true;
  }
  error.value = false;
  try {
    const page = await client.getClans(PAGE_SIZE, offset, searchTerm || undefined);
    if (offset === 0) {
      clans.value = page.clans;
    } else {
      clans.value = [...clans.value, ...page.clans];
    }
    total.value = page.total;
  } catch {
    error.value = true;
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
}

function loadMore() {
  void loadClans(clans.value.length, search.value, clans.value.length > 0);
}

watch(search, (value) => {
  if (searchDebounce) {
    clearTimeout(searchDebounce);
  }
  searchDebounce = setTimeout(() => {
    void loadClans(0, value, false);
  }, SEARCH_DEBOUNCE_MS);
});

onBeforeUnmount(() => {
  if (searchDebounce) {
    clearTimeout(searchDebounce);
  }
});

void loadClans(0, "", false);

useHead({
  title: `Clans - TrackScape`,
});
</script>

<template>

  <PageTitle title="Clans">
    <input
      v-model="search"
      type="text"
      placeholder="Type clan name here"
      class="input w-full md:max-w-md max-w-full" />
  </PageTitle>


  <div v-if="loading"
       class="flex justify-center py-16">
    <span class="loading loading-spinner loading-lg"></span>
  </div>

  <div v-else-if="error"
       role="alert"
       class="alert alert-error">
    <span>Failed to load clans.</span>
    <div>
      <button class="btn btn-sm"
              @click="loadMore">Try again</button>
    </div>
  </div>

  <template v-else>
    <div v-if="clans.length === 0"
         class="py-16 text-center opacity-70">
      <p>No clans found<template v-if="search"> for "{{search}}"</template>.</p>
    </div>

    <TransitionGroup name="list"
                     tag="div"
                     class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div v-for="clan in clans"
           :key="clan.id">
        <div class="card bg-primary text-primary-content card-sm shadow-xs shadow-accent-content">
          <div class="card-body">
            <h2 class="card-title">{{clan.name}}
            </h2>
            <span class="text-sm">{{clan.registered_members}} clanmates</span>

            <div class="card-actions justify-end">
              <router-link :to="{name: 'members', params: {clanId: clan.id}}"
                           class="btn ">View</router-link>

            </div>
          </div>
        </div>
      </div>
    </TransitionGroup>

    <div v-if="hasMore || loadingMore"
         class="flex flex-col items-center gap-2 py-8">
      <span class="text-sm opacity-70">Loaded {{clans.length}} of {{total}} clans</span>
      <button class="btn btn-primary"
              :disabled="loadingMore"
              @click="loadMore">
        <span v-if="loadingMore"
              class="loading loading-spinner"></span>
        Load more
      </button>
    </div>
  </template>

</template>

<style scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.5s ease;
}
.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
