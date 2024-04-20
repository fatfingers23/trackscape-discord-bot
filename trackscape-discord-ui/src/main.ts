import './assets/main.css'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createHead } from '@unhead/vue'
import App from './App.vue'
import router from './router'

const app = createApp(App)

const head = createHead()
app.use(head)
app.use(createPinia())
app.use(router)

app.mount('#app')
