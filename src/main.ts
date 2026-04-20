import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./style.css";
import { useAiStore } from "./stores/ai";
import { useThemeStore } from "./stores/theme";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

// 启动时初始化 Store
useAiStore(pinia).loadConfig();
useThemeStore(pinia); // 实例化以触发主题应用

app.mount("#app");
