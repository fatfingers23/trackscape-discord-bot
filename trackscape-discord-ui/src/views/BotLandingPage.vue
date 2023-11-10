<script setup lang="ts">

import {ref} from "vue";
import TrackscapeApiClient, {BotInfo} from "@/services/TrackscapeApiClient";

let client = new TrackscapeApiClient(import.meta.env.VITE_API_BASE_URL);

let botInfo = ref<BotInfo>({
  server_count: 0,
  connected_users: 0
});

client.getBotInfo().then((info) => {
  botInfo.value = info as BotInfo;
});


</script>

<template>

  <div class="container mx-auto p-10 min-h-screen flex flex-col justify-center">
    <!-- Header Section -->
    <div class="text-center my-16 ">
      <img
        class="object-cover object-center w-24 h-24 mx-auto mb-4 rounded-full"
        src="@/assets/img/Trackscape_Logo_icon.png"
      >
      <h1 class="text-4xl font-bold mb-3">
        TrackScape
      </h1>

      <p
        class=" mb-8"
      >
        TrackScape is a Discord bot that allows you to connect in ways never before possible with Discord and your
        OSRS clan.
      </p>
      <a
        href="https://discord.com/api/oauth2/authorize?client_id=864626697327869952&permissions=2147568704&scope=bot"
        target="_blank"
        class="btn btn-primary btn-outline"
      >Invite to Discord <img
        class="w-7 h-8 ml-2"
        src="@/assets/img/icon_clyde_blurple_RGB.svg"
        alt="Discord Logo"
      > </a>
      <div class="mt-4 max-w-50">
        <div class="stats ">
          <div class="stat">
            <div class="stat-figure text-secondary">
              <img
                class="w-7 h-8 ml-2"
                src="@/assets/img/icon_clyde_blurple_RGB.svg"
                alt="Discord Logo"
              >
            </div>
            <div class="stat-title">
              Servers Joined
            </div>
            <div class="stat-value">
              {{ botInfo.server_count.toLocaleString() }}
            </div>
          </div>

          <div class="stat">
            <div class="stat-figure text-secondary">
              <img
                src="https://oldschool.runescape.wiki/images/Your_Clan_icon.png"
                alt="In game cc icon"
              >
            </div>
            <div class="stat-title">
              Scapers Chatting
            </div>
            <div class="stat-value text-secondary">
              {{ botInfo.connected_users.toLocaleString()}}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="p-x3">
      <!-- Features Section -->
      <h2 class="text-3xl font-bold mb-8 text-center">
        Features
      </h2>

      <div class="grid grid-cols-1 md:grid-cols-1 lg:grid-cols-2 gap-8">
        <div class="card bordered glass">
          <figure>
            <img
              src="@/assets/img/chat_from_cc.png"
              alt="Pic of the feature of getting cc in discord"
            >
          </figure>
          <div class="card-body">
            <h2 class="card-title">
              Live CC in Discord!
            </h2>
            <p>Get in game clan chat sent to a channel of your choosing!.</p>
          </div>
        </div>

        <div class="card bordered glass">
          <figure>
            <img
              src="@/assets/img/discord_to_chat.png"
              alt="Pic of the feature of sending discord messages to cc"
            >
          </figure>
          <div class="card-body">
            <h2 class="card-title">
              Not a one way road!
            </h2>
            <p>Send messages from discord directly to in game clan chat</p>
          </div>
        </div>

        <div class="card bordered glass">
          <figure>
            <img
              src="@/assets/img/raid_drop_broadcast.png"
              alt="Styled broadcast for drops"
            >
          </figure>
          <div class="card-body">
            <h2 class="card-title">
              Embded Broadcasts
            </h2>
            <p>Get style messages of drops, quest completion, pet drops, and more!</p>
          </div>
        </div>

        <div class="card bordered glass">
          <figure>
            <img
              src="@/assets/img/GitHub_Logo_White.png"
              alt="Github Logo"
            >
          </figure>
          <div class="card-body">
            <h2 class="card-title">
              Got an idea?
            </h2>
            <p>Have an idea you'd like to see? Add an issue requesting it</p>
            <div class="card-actions">
              <a
                class="btn btn-primary btn-outline"
                href="https://github.com/fatfingers23/trackscape-discord-bot/issues"
              >
                Submit an Issue
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

</template>
