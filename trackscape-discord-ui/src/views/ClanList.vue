<script setup lang="ts">
import {ref} from "vue";

type Clans = {
  id: string,
  name: string
}

let clans = ref<Clans[]>([]);

fetch("/api/clans/list").then((res) => {
  res.json().then((data) => {
    clans.value = data;
  });
});



</script>

<template>
  <div class="container mx-auto p-10 min-h-screen flex flex-col ">
    <div class="overflow-x-auto">
      <table class="table">
        <!-- head -->
        <thead>
          <tr>
            <th>Name</th>
          </tr>
        </thead>
        <tbody>

          <tr v-for="(clan, index) in clans"
              :key="index">
            <th>{{clan.name}}</th>

          </tr>

        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>

</style>
