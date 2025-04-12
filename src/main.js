import { createApp } from "vue";
import App from "./App.vue";
import Toast, { POSITION } from "vue-toastification";
import { createVfm } from "vue-final-modal";
import FloatingVue from "floating-vue";

import "vue-toastification/dist/index.css";
import "vue-final-modal/style.css";
import "floating-vue/dist/style.css";
import "./style.css";

import { VueFinalModal } from "vue-final-modal";
import BaseModal from "@/components/common/BaseModal.vue";

const vfm = createVfm();

const app = createApp(App)
  .component("VueFinalModal", VueFinalModal)
  .component("BaseModal", BaseModal);

app.use(Toast, {
  position: POSITION.BOTTOM_RIGHT,
  timeout: 5000,
  transition: "Vue-Toastification__fade",
  toastClassName: "lrcget-toast",
  bodyClassName: ["toast-body-1", "toast-body-2"],
  hideProgressBar: true,
  closeButton: false,
  draggablePercent: 0.4,
  closeOnClick: false,
});
app.use(FloatingVue, {
  themes: {
    "lrcget-tooltip": {
      $extend: "tooltip",
      delay: {
        show: 50,
        hide: 50,
      },
    },
  },
});
app.use(vfm);
app.mount("#app");

document.addEventListener(
  "contextmenu",
  (event) => {
    event.preventDefault();
    return false;
  },
  { capture: true },
);
