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
    },
    title: {
      type: String,
      required: false,
      default: ""
    },
    description: {
      type: String,
      required: false,
      default: ""
    },
    noResultsMessage: {
      type: String,
      required: false,
      default: "No results found"
    }
  });

  let searchedTerm = ref<string>("");
  let leaderBoardTitle = ref<string>(props.title);


  let filteredData = computed(() => {
    return props.data.filter((item) => {
      if (props.searchField === "") {
        return true;
      }
      let value = '';
      if(props.searchField.includes('.')){
        value = getMultiLevelValue(item, props.searchField);
      } else {
        value = item[props.searchField];
      }

      if(value === undefined){
        throw new Error(`Could not find the key ${props.searchField} in the data to search by`);
      }

      return value.toLowerCase().includes(searchedTerm.value.toLowerCase());
    });
  });

  const getMultiLevelValue = (obj: any, key: string) => {
    const keys = key.split('.');
    let value = obj;
    for (let i = 0; i < keys.length; i++) {
      value = value[keys[i]];
    }
    return value;
  };

</script>

<template>
  <div>
    <div class="flex justify-between items-center pb-2">
      <div class="flex flex-col md:w-1/2 w-full pb-2">
        <h3 v-if="props.title !== ''"
            class="text-lg font-medium text-neutral-content pb-1">{{leaderBoardTitle}}</h3>
        <p v-if="props.description !== ''"
           class="text-sm">{{props.description}}</p>
        <input
          v-if="props.searchField !== ''"
          v-model="searchedTerm"
          type="text"
          placeholder="Search"
          class="input input-bordered w-full md:max-w-md max-w-full mb-3 mt-2" />
      </div>
    </div>
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
          <td :colspan="props.columns?.length">{{ props.noResultsMessage }}</td>
        </tr>

        <tr v-for="(item, index) in filteredData"
            class="hover"
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
