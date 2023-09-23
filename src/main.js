import { createApp } from 'vue'
import 'overlayscrollbars/css/OverlayScrollbars.css'
import App from "./App.vue"
import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
import Toast, { POSITION } from 'vue-toastification'
import 'vue-toastification/dist/index.css'
import './style.css'

const app = createApp(App)
  .component('OverlayScrollbars', OverlayScrollbarsComponent)

app.use(Toast, {
  position: POSITION.BOTTOM_RIGHT,
  timeout: 50000,
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
