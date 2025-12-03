import { createApp } from "vue";
import { createPinia } from "pinia";
import "./style.css";
import App from "./App.vue";
import router from "./router";

const app = createApp(App);

app.use(createPinia());
app.use(router);

// Disable context menu globally
window.addEventListener('contextmenu', (e) => {
  e.preventDefault();
});

// Prevent mouse thumb buttons (forward/back) from navigating history
window.addEventListener('mouseup', (e) => {
  if (e.button === 3 || e.button === 4) {
    e.preventDefault();
  }
});

app.mount("#app");
