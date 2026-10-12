<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, vec2, xy} from './geometry.js'

const { isDark } = useData()

const A = vec2(2, 2), B = vec2(8, 4), P = vec2(4, 7)

const u = B.sub(A)
const v = P.sub(A)

const uNormalizado = u.normalized()
const uLinha = vec2(-u.y, u.x)
const uLinhaNormalizado = uLinha.normalized()

const baseDaPerpendicular = A.add(u.mul(v.dot(u) / u.dot(u)))

const edge = computed(() => {
  const gap = 0.42
  const cores = palette(isDark.value)

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  return {
    data: [
      // a reta infinita que passa por A e B
      linha(
        [A.sub(uNormalizado.mul(1.2)), B.add(uNormalizado.mul(1.2))],
        { line: { color: cores.fraca, width: 1, dash: 'dot' } }
      ),
      // o triângulo ABP
      linha(
        [A, B, P, A],
        { fill: 'toself', fillcolor: cores.preenche, line: { color: cores.fraca, width: 1 } }
      ),
      // a altura h, medida na direção de u'
      linha(
        [baseDaPerpendicular, P],
        { line: { color: cores.tinta, width: 1.5, dash: 'dash' } }
      ),
      // o ângulo reto da perpendicular
      linha(
        [uNormalizado, uNormalizado.add(uLinhaNormalizado), uLinhaNormalizado].map((d) => baseDaPerpendicular.add(d.mul(0.38))), 
        { line: { color: cores.fraca, width: 1.2 } }
      ),
      {
        ...xy([A, B, P]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cores.tinta },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy([
          A.nudged_right(-0.42),
          B.nudged_right(0.42), 
          P.nudged_up(0.3),
          u.label(A, -gap),
          v.label(A, gap),
          uLinha.label(A, gap),
          P.sub(baseDaPerpendicular).label(baseDaPerpendicular, -gap),
        ]),
        text: ['A', 'B', 'P', 'u', 'v', "u'", 'h'],
        textposition: ['middle left', 'middle right', 'top center',
          'middle center', 'middle center', 'middle center', 'middle center'],
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 15 },
        hoverinfo: 'skip', showlegend: false,
      },
    ],
    layout: {
      ...eixosIguais([-1.4, 9.4], [0.6, 9.2]),
      annotations: [
        seta(A, B, cores.serie[0]),
        seta(A, P, cores.serie[1]),
        seta(A, A.add(uLinha), cores.serie[2]),
      ],
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="edge.data" :layout="edge.layout" />
</template>
