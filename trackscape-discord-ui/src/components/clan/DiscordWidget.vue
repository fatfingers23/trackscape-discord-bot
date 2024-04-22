<script setup lang="ts">

import {ref} from "vue";

const props = defineProps({
  discord_id: {
    type: String,
    required: true
  }
})


type DiscordWidgetResponse = {
  id: string
  name: string
  instant_invite: string
  presence_count: number
}

let discordWidget = ref<DiscordWidgetResponse>();

fetch(`https://discord.com/api/guilds/${props.discord_id}/widget.json`)
  .then((response) => {
    if (!response.ok) {
      //No major issue, if they dont have widgets installed they dont have widgets
      return;
    }

    return response.json() as Promise<DiscordWidgetResponse>
  })
  .then((data) => {
    discordWidget.value = data
  })

</script>

<template>
  <Transition name="slide-fade">
    <div
      v-if="discordWidget !== undefined"
      class="cursor-default select-none space-y-2 rounded-sm bg-[#f2f3f5] p-4 dark:bg-[#2b2d31] shadow-xl">
      <p class="text-xs font-semibold uppercase text-[#4e5058] dark:text-[#b5bac1]">You've been invited to join a server</p>
      <div class="flex items-center justify-between gap-16">
        <div class="flex items-center gap-4">
          <img src="https://cdn.discordapp.com/embed/avatars/0.png?size=128"
               alt="Discord"
               class="h-14 w-14 rounded-xl"
               draggable="false" />
          <div>
            <a target="_blank"
               rel="noopener noreferrer"
               href="https://discord.com"><h1 class="cursor-pointer font-normal text-[#060607] hover:underline dark:text-white">
                 {{ discordWidget.name }}</h1></a>
            <div class="flex items-center justify-between gap-3 text-xs">
              <p class="text-[#80848e]">
                <span class="inline-flex"
                ><svg class="h-[0.6rem] w-[0.6rem] fill-[#23a559]"
                      stroke-width="0"
                      viewBox="0 0 512 512"
                      xmlns="http://www.w3.org/2000/svg"><path d="M256 23.05C127.5 23.05 23.05 127.5 23.05 256S127.5 488.9 256 488.9 488.9 384.5 488.9 256 384.5 23.05 256 23.05z"></path></svg
                ></span>
                {{discordWidget.presence_count}} Online
              </p>
              <p class="text-[#80848e]">
                <span class="inline-flex"></span>
              <!--              ><svg class="h-[0.6rem] w-[0.6rem] fill-[#b5bac1] dark:fill-[#4e5058]"-->
              <!--                    stroke-width="0"-->
              <!--                    viewBox="0 0 512 512"-->
              <!--                    xmlns="http://www.w3.org/2000/svg"><path d="M256 23.05C127.5 23.05 23.05 127.5 23.05 256S127.5 488.9 256 488.9 488.9 384.5 488.9 256 384.5 23.05 256 23.05z"></path></svg-->
              <!--              ></span>-->
              <!--              3,632 Members-->
              </p>
            </div>
          </div>
        </div>
        <a target="_blank"
           rel="noopener noreferrer"
           :href="discordWidget.instant_invite"><button class="focus-visible:ring-ring ring-offset-background inline-flex h-10 items-center justify-center rounded-md bg-[#248046] px-4 py-2 text-sm font-medium text-[#e9ffec] transition-colors hover:bg-[#1a6334] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50">Join</button></a>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.slide-fade-enter-active {
  transition: all 0.3s ease-in;
}

.slide-fade-leave-active {
  transition: all 0.8s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(20px);
  opacity: 0;
}
</style>
