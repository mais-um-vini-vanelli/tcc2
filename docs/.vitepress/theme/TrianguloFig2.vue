<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, linha2d, pontos2d, rotulos2d, alfa, vec2 } from './geometry.js'

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

  const rotulos = [
    { text: 'A', position: A.nudge(-0.42, 0), textposition: 'middle left' },
    { text: 'B', position: B.nudge(0.42, 0), textposition: 'middle right' },
    { text: 'C', position: C.nudge(0, 0.3), textposition: 'top center' },
    { text: 'P', position: P.nudge(-0.3, 0), textposition: 'top center' },
    ...regioes.map((r) => ({ text: r.nome, position: centro(...r.pontos) })),
  ]

  return {
    data: [
      // os subtriângulos sem contorno próprio
      ...regioes.map((r, i) => linha2d(
        [...r.pontos, r.pontos[0]],
        { fill: 'toself', fillcolor: alfa(cores.serie[i], 0.25), mode: 'none' }
      )),
      // o contorno de ABC
      linha2d([A, B, C, A], { line: { color: cores.fraca, width: 1.4 } }),
      // os raios que saem de P
      ...[A, B, C].map((vertice) =>
        linha2d([vertice, P], { line: { color: cores.fraca, width: 1 } })
      ),
      pontos2d([A, B, C, P], cores.tinta),
      rotulos2d(rotulos, cores.tinta, 15),
    ],
    layout: cena2d([0.9, 9.1], [1.0, 8.2]),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" />
</template>
