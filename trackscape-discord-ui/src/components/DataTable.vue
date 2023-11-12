<script setup lang="ts">
  import type {PropType} from "vue";

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

  })
</script>

<template>
  <table class="table">
    <thead>
      <tr>
        <th v-for="(column, index) in props.columns"
            :key="index">{{ column.name }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(item, index) in data"
          :key="index">
        <th v-for="(column, index) in props.columns"
            :key="index">
          <slot name="row-item"
                :column="column"
                :item="item">
            {{ item[column.key] }}
          </slot>
        </th>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>

</style>
