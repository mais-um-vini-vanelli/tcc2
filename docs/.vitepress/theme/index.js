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
import EsferaRaio from './EsferaRaio.vue'
import TrianguloE1E2 from './TrianguloE1E2.vue'
import VolumeDeterminante from './VolumeDeterminante.vue'
import ParalelepipedoDet from './ParalelepipedoDet.vue'
import CameraInclinada from './CameraInclinada.vue'
import ConeDoSpot from './ConeDoSpot.vue'
import TelaVirtual from './TelaVirtual.vue'
import BvhCena from './BvhCena.vue'
import TrianguloWinding from './TrianguloWinding.vue'
import RaioEspacoLocal from './RaioEspacoLocal.vue'
import AreaAparente from './AreaAparente.vue'
import NormalEscalada from './NormalEscalada.vue'
import RaioDeSombra from './RaioDeSombra.vue'
import NivelDoMipmap from './NivelDoMipmap.vue'

import './style.css'

function marcarItemAtivo() {
    const links = document.querySelectorAll('.VPLocalNavOutlineDropdown .outline-link')
    if (!links.length) return

    let ativo = null
    for (const link of links) {
        link.classList.remove('active')
        const alvo = document.getElementById(decodeURIComponent(link.hash.slice(1)))
        if (alvo && alvo.getBoundingClientRect().top <= 100) ativo = link
    }
    ativo?.classList.add('active')
}

if (typeof window !== 'undefined') {
    const agendar = () => requestAnimationFrame(marcarItemAtivo)
    document.addEventListener('click', agendar, true)
    window.addEventListener('scroll', agendar, { passive: true })
}

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
        app.component('EsferaRaio', EsferaRaio)
        app.component('TrianguloE1E2', TrianguloE1E2)
        app.component('VolumeDeterminante', VolumeDeterminante)
        app.component('ParalelepipedoDet', ParalelepipedoDet)
        app.component('CameraInclinada', CameraInclinada)
        app.component('ConeDoSpot', ConeDoSpot)
        app.component('TelaVirtual', TelaVirtual)
        app.component('BvhCena', BvhCena)
        app.component('TrianguloWinding', TrianguloWinding)
        app.component('RaioEspacoLocal', RaioEspacoLocal)
        app.component('AreaAparente', AreaAparente)
        app.component('NormalEscalada', NormalEscalada)
        app.component('RaioDeSombra', RaioDeSombra)
        app.component('NivelDoMipmap', NivelDoMipmap)
    }
}
