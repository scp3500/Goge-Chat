import * as PIXI from 'pixi.js'
// @ts-ignore
window.PIXI = PIXI

// @ts-ignore
window.url = {
    resolve: (from: string, to: string) => {
        try {
            return new URL(to, new URL(from, window.location.href).href).href;
        } catch (e) {
            return to;
        }
    }
}

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './assets/main.css'
import App from './App.vue'

const app = createApp(App)
const pinia = createPinia() // 创建实例

app.use(pinia) // 【关键】必须在 mount 之前注册 Pinia
app.mount('#app')