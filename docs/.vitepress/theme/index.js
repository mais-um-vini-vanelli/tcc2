import DefaultTheme from 'vitepress/theme'
import Plot from './Plot.vue'
import TrianguloFig1 from './TrianguloFig1.vue'
import TrianguloFig2 from './TrianguloFig2.vue'
import TrianguloFig3 from './TrianguloFig3.vue'
import IntersecoesFig4 from './IntersecoesFig4.vue'
import TransformFig1 from './TransformFig1.vue'
import FovMeiaAltura from './FovMeiaAltura.vue'
import BvhFig2 from './BvhFig2.vue'
import ProjecaoNdc from './ProjecaoNdc.vue'
import GamaCurva from './GamaCurva.vue'
import PlanoDistancia from './PlanoDistancia.vue'
import ReflexaoVetor from './ReflexaoVetor.vue'
import RefracaoSnell from './RefracaoSnell.vue'
import EsferaPhi from './EsferaPhi.vue'
import LeiCossenoFeixe from './LeiCossenoFeixe.vue'
import EscalaRotacaoShear from './EscalaRotacaoShear.vue'

export default {
    extends: DefaultTheme,
    enhanceApp({ app }) {
        app.component('Plot', Plot)
        app.component('FovMeiaAltura', FovMeiaAltura)
        app.component('TrianguloFig1', TrianguloFig1)
        app.component('TrianguloFig2', TrianguloFig2)
        app.component('TrianguloFig3', TrianguloFig3)
        app.component('IntersecoesFig4', IntersecoesFig4)
        app.component('TransformFig1', TransformFig1)
        app.component('BvhFig2', BvhFig2)
        app.component('ProjecaoNdc', ProjecaoNdc)
        app.component('GamaCurva', GamaCurva)
        app.component('PlanoDistancia', PlanoDistancia)
        app.component('ReflexaoVetor', ReflexaoVetor)
        app.component('RefracaoSnell', RefracaoSnell)
        app.component('EsferaPhi', EsferaPhi)
        app.component('LeiCossenoFeixe', LeiCossenoFeixe)
        app.component('EscalaRotacaoShear', EscalaRotacaoShear)
    }
}
