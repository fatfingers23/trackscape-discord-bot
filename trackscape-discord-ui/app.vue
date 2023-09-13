<script setup lang="ts">
useHead({
  htmlAttrs: {
    'data-theme': 'dracula'
  }
})

</script>

<template>
  <div data-theme="dracula">
    <NuxtLayout>
      <NuxtPage />
    </Nuxtlayout>
  </div>
</template>
