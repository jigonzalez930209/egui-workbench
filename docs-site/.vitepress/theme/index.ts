import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'

import ApiTable from './components/ApiTable.vue'
import LiveDemo from './components/LiveDemo.vue'
import Preview from './components/Preview.vue'
import TokenSwatch from './components/TokenSwatch.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('ApiTable', ApiTable)
    app.component('LiveDemo', LiveDemo)
    app.component('Preview', Preview)
    app.component('TokenSwatch', TokenSwatch)
  },
} satisfies Theme
