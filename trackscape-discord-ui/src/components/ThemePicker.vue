<script setup lang="ts">
  import {ref} from "vue";

  const themes = [
    "default",
    "light",
    "dark",
    "cupcake",
    "bumblebee",
    "emerald",
    "corporate",
    "synthwave",
    "retro",
    "cyberpunk",
    "valentine",
    "halloween",
    "garden",
    "forest",
    "aqua",
    "lofi",
    "pastel",
    "fantasy",
    "wireframe",
    "black",
    "luxury",
    "dracula",
    "cmyk",
    "autumn",
    "business",
    "acid",
    "lemonade",
    "night",
    "coffee",
    "winter",
    "dim",
    "nord",
    "sunset"
  ];

  let savedTheme = localStorage.getItem("theme");
  let selectedTheme =  ref(savedTheme ?? "default");

  const onSelect = ((theme: string) => {
    localStorage.setItem("theme", theme);
  });


  //HACK Does not seem typescripty
  window.addEventListener('click', function(e) {
    document.querySelectorAll('.theme-picker').forEach(function(dropdown) {
      //@ts-ignore
      if (!dropdown.contains(e.target )) {
        //@ts-ignore
        dropdown.open = false;
      }
    });
  });

</script>

<template>
  <details class="theme-picker">
    <summary>
      Pick Theme
    </summary>
    <ul class="p-2 bg-base-100">
      <li v-for="theme in themes"
          :key="theme">

        <input
          v-model="selectedTheme"
          @click="onSelect(theme)"
          type="radio"
          name="theme-dropdown"
          :class="['theme-controller btn btn-sm btn-block btn-ghost justify-start ']"
          :aria-label="`${theme == savedTheme ? 'x - ': ''} ${theme}`"
          :checked="theme === selectedTheme"
          :value="theme"/>
      </li>
    </ul>
  </details>
</template>

<style scoped>

.theme-picker {
  z-index: 10000000;
}

</style>
