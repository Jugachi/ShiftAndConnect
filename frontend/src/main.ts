import { createApp } from 'vue'
import App from './App.vue'
import router from './router' // <-- WICHTIG 1: Router importieren

// Vuetify Setup
import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'dark'
  }
})

const app = createApp(App)

// WICHTIG 2: Diese Zeile füttert die ganze App mit dem Router!
// Fehlt diese Zeile, wirft Vue exakt deine Fehlermeldung.
app.use(router)  
app.use(vuetify)

app.mount('#app')