import { createApp } from "vue";
import "./style.css";
import "overlayscrollbars/css/OverlayScrollbars.css";
import App from "./App.vue";
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";

createApp(App)
  .component("OverlayScrollbars", OverlayScrollbarsComponent)
  .mount("#app")

document.addEventListener(
  "contextmenu",
  (event) => {
    event.preventDefault();
    return false;
  },
  { capture: true }
)
