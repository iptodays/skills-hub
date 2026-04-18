import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./style.css";
import { useAiStore } from "./stores/ai";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

// 启动时从配置文件加载 AI 设置
useAiStore(pinia).loadConfig();

app.mount("#app");
