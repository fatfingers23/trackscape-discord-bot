// https://nuxt.com/docs/api/configuration/nuxt-config
import path from 'path'

export default defineNuxtConfig({
  devtools: { enabled: true },
  nitro: {
    output: {
      publicDir: path.join(__dirname, '../trackscape-discord-api/ui')
    }
  },
  modules: [
    '@nuxtjs/tailwindcss'
    // '@nuxtjs/eslint-module'
  ]

})
