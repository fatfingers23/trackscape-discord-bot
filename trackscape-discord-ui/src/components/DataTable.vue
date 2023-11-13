<script setup lang="ts">
  import type {PropType} from "vue";
  import {computed, ref} from "vue";

  type column = {
    name: string,
    key: string
  }

  const props = defineProps({
    data: {
      type: Array as PropType<any[]>,
      required: true
    },
    columns: {
      type: Array as PropType<column[]>,
      required: true
    },
    searchField: {
      type: String,
      required: false,
      default: ""
    }
  })

  let searchedTerm = ref<string>("");


  let filteredData = computed(() => {
    return props.data.filter((item) => {
      if (props.searchField === "") {
        return true;
      }
      return item[props.searchField].toLowerCase().includes(searchedTerm.value.toLowerCase());
    });
  });


</script>

<template>
  <div>
    <input
      v-if="props.searchField !== ''"
      v-model="searchedTerm"
      type="text"
      placeholder="Search"
      class="input input-bordered w-full md:max-w-md max-w-full mb-3" />
    <table class="table">
      <thead>
        <tr>
          <th v-for="(column, index) in props.columns"
              :key="index">{{ column.name }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="filteredData.length === 0"
            class="text-center min-w-full">
          <td><p>No results found</p></td>
        </tr>

        <tr v-for="(item, index) in filteredData"
            :key="index">
          <th v-for="(column, index) in props.columns"
              :key="index">
            <slot name="row-item"
                  :column="column"
                  :item="item"
                  :index="index"
            >
              {{ item[column.key] }}
            </slot>
          </th>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>

</style>
