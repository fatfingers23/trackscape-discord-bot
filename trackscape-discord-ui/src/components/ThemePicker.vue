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
    "sunset",
    "caramellatte",
    "abyss",
    "silk"
  ];

  const savedTheme = localStorage.getItem("theme");
  const selectedTheme = ref(savedTheme ?? "default");

  // This component is mounted more than once at the same time (the drawer
  // sidebar and the top navbar both render it), so the popover id and CSS
  // anchor name have to be unique per instance.
  const uid = `theme-picker-${Math.random().toString(36).slice(2, 9)}`;
  const popoverId = `${uid}-popover`;
  const radioGroupName = `${uid}-theme`;
  const anchorName = `--${uid}`;

  const themePopover = ref<HTMLElement | null>(null);

  const onSelect = (theme: string) => {
    localStorage.setItem("theme", theme);
    themePopover.value?.hidePopover();
  };
</script>

<template>
  <button type="button"
          :popovertarget="popoverId"
          :style="`anchor-name: ${anchorName}`">
    Pick Theme
  </button>

  <!--
    Teleported out of the menu <ul> so daisyUI's submenu styles do not apply to
    the popover. Anchor positioning is document-wide, so it still anchors to
    the button above.
  -->
  <Teleport to="body">
    <ul ref="themePopover"
        :id="popoverId"
        popover
        class="dropdown dropdown-end bg-base-100 rounded-box z-1 w-52 p-2 shadow-sm max-h-[70vh] overflow-y-auto"
        :style="`position-anchor: ${anchorName}`">
      <li v-for="theme in themes"
          :key="theme">
        <input v-model="selectedTheme"
               @click="onSelect(theme)"
               type="radio"
               :name="radioGroupName"
               class="theme-controller w-full btn btn-sm btn-block btn-ghost justify-start"
               :aria-label="`${theme == savedTheme ? 'x - ': ''} ${theme}`"
               :value="theme" />
      </li>
    </ul>
  </Teleport>
</template>
