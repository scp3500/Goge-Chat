// src/main.js
import { createApp } from 'vue'
import { createPinia } from 'pinia' // 必须导入
import './assets/main.css'
import App from './App.vue'

const app = createApp(App)
const pinia = createPinia() // 创建实例

app.use(pinia) // 【关键】必须在 mount 之前注册 Pinia
app.mount('#app')