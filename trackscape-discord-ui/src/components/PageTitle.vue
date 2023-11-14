<script setup lang="ts">
const props = defineProps({
  title: {
    type: String,
    required: true
  }
})

</script>

<template>

  <div class="text-left md:my-8 my-4 ">
    <h1 class="text-4xl font-bold mb-3">
      {{props.title}}
    </h1>
    <div class="divider divider-accent"></div>
    <slot></slot>
  </div>
</template>

<style scoped>

</style>
