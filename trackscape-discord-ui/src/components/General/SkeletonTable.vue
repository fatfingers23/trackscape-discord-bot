<script setup lang="ts">

const props = defineProps({
  rows: {
    type: Number,
    default: 10,
    required: false
  },
  columns: {
    type: Number,
    required: true
  },
  searchField: {
    type: Boolean,
    required: false,
    default: false
  }
});

</script>

<template>
  <div class="flex flex-col gap-2 w-full">
    <div
      v-if="props.searchField"
      type="text"
      class="skeleton w-full md:max-w-md max-w-full mb-3 h-16" />
    <div class="">
      <table class="table ">
        <thead>
          <tr class="skeleton">
            <th class="h-12"
                v-for="(index) in props.columns"
                :key="index"> </th>
          </tr>
        </thead>
        <tbody class="mt-10">
          <tr class="skeleton"
              v-for="(index) in props.rows"
              :key="index">
            <th
              class=" h-10 text-base "
              v-for="(index) in props.columns"
              :key="index">

            </th>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

</template>

<style scoped>

</style>
