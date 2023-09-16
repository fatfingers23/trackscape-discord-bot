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
    <footer class="footer p-10 bg-base-200 text-base-content">
      <aside class="items-center grid-flow-col">
        <img
          src="/img/Trackscape_Logo_icon.png"
          alt="Trackscape Logo"
          width="24"
          height="24"
          class="fill-current rounded-full"
        >
        <p>TrackScape<br>Scaping and Tracking</p>
      </aside>
      <nav class="md:place-self-center md:justify-self-end">
        <div class="grid grid-flow-col gap-4">
          <a href="https://github.com/fatfingers23/trackscape-discord-bot">
            <img
              src="/img/github-mark-white.svg"
              alt="GitHub Logo"
              width="24"
              height="24"
              class="fill-current"
            >

          </a>
        </div>
      </nav>
    </footer>
  </div>
</template>
