<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, alfa, vec2, xy } from './geometry.js'

const { isDark } = useData()

const A = vec2(2, 2), B = vec2(8, 4), C = vec2(4, 7)
const P = vec2(5, 4)

const centro = (...pontos) =>
  pontos.reduce((soma, p) => soma.add(p), vec2(0, 0)).mul(1 / pontos.length)

const regioes = [
  { nome: 'BCP', pontos: [B, C, P] },
  { nome: 'CAP', pontos: [C, A, P] },
  { nome: 'ABP', pontos: [A, B, P] },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  return {
    data: [
      // os subtriângulos sem contorno próprio
      ...regioes.map((r, i) => linha(
        [...r.pontos, r.pontos[0]],
        { fill: 'toself', fillcolor: alfa(cores.serie[i], 0.25), mode: 'none' }
      )),
      // o contorno de ABC
      linha([A, B, C, A], { line: { color: cores.fraca, width: 1.4 } }),
      // os raios que saem de P
      ...[A, B, C].map((vertice) =>
        linha([vertice, P], { line: { color: cores.fraca, width: 1 } })
      ),
      {
        ...xy([A, B, C, P]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cores.tinta },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy([
          A.nudged_right(-0.42),
          B.nudged_right(0.42),
          C.nudged_up(0.3),
          P.nudged_right(-0.3),
          ...regioes.map((r) => centro(...r.pontos)),
        ]),
        text: ['A', 'B', 'C', 'P', ...regioes.map((r) => r.nome)],
        textposition: ['middle left', 'middle right', 'top center', 'top center',
          'middle center', 'middle center', 'middle center'],
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 15 },
        hoverinfo: 'skip', showlegend: false,
      },
    ],
    layout: {
      ...eixosIguais([0.9, 9.1], [1.0, 8.2]),
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" />
</template>
