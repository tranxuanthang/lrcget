import { createApp } from 'vue'
import App from "./App.vue"
import Toast, { POSITION } from 'vue-toastification'
import 'vue-toastification/dist/index.css'
import './style.css'

// Directive that listens for clicks outside a components, useful for popups
// Source: https://stackoverflow.com/a/64698630
const clickOutside = {
  beforeMount: (el, binding) => {
    el.clickOutsideEvent = event => {
      // here I check that click was outside the el and his children
      if (!(el == event.target || el.contains(event.target))) {
        // and if it did, call method provided in attribute value
        binding.value(event);
      }
    };
    document.addEventListener("click", el.clickOutsideEvent);
    document.addEventListener("touchstart", el.clickOutsideEvent);
  },
  unmounted: el => {
    document.removeEventListener("click", el.clickOutsideEvent);
    document.removeEventListener("touchstart", el.clickOutsideEvent);
  },
};


const app = createApp(App)
  .directive("click-outside", clickOutside)

app.use(Toast, {
  position: POSITION.BOTTOM_RIGHT,
  timeout: 5000,
  transition: 'Vue-Toastification__fade',
  toastClassName: 'lrcget-toast',
  bodyClassName: ['toast-body-1', 'toast-body-2'],
  hideProgressBar: true,
  closeButton: false,
  draggablePercent: 0.4,
  closeOnClick: false
})
app.mount('#app')

document.addEventListener(
  'contextmenu',
  (event) => {
    event.preventDefault()
    return false
  },
  { capture: true }
)
