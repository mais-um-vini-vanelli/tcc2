import DefaultTheme from 'vitepress/theme'
import Plot from './Plot.vue'

export default {
    extends: DefaultTheme,
    enhanceApp({ app }) {
        app.component('Plot', Plot)
    }
}