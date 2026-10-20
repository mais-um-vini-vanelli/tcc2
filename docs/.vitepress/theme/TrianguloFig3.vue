<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, linha2d, pontos2d, rotulos2d, alfa, vec2 } from './geometry.js'

const { isDark } = useData()

const A = vec2(2.5, 2), B = vec2(7.5, 4), C = vec2(4, 6.5)

// os subtriângulos de P, na mesma ordem de TrianguloFig2
const regioes = (P) => [[B, C, P], [C, A, P], [A, B, P]]

const casos = [
  {
    titulo: 'P em um vértice',
    P: A,
    pesosBC: ["0", "0"],
    rotuloA: (a) => ({ text: 'A = P (1)', position: a.nudge(0, -0.30), textposition: 'bottom center' }),
    rotuloP: false,
  },
  {
    titulo: 'P no meio de AB',
    P: vec2(5, 3),
    pesosBC: ["½", "0"],
    rotuloA: (a) => ({ text: 'A (½)', position: a.nudge(-0.42, 0), textposition: 'middle left' }),
    rotuloP: true,
  },
  {
    titulo: 'P no centro',
    P: vec2(14 / 3, 13 / 3),
    pesosBC: ["⅓", "⅓"],
    rotuloA: (a) => ({ text: 'A (⅓)', position: a.nudge(-0.42, 0), textposition: 'middle left' }),
    rotuloP: true,
  },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const painel = (caso, index) => {
    const offset = vec2(index * 8.6, 0)
    const [a, b, c, p] = [A, B, C, caso.P].map((v) => v.add(offset))

    const rotulos = [
      caso.rotuloA(a),
      { text: `B (${caso.pesosBC[0]})`, position: b.nudge(0.42, 0), textposition: 'middle right' },
      { text: `C (${caso.pesosBC[1]})`, position: c.nudge(0, 0.3), textposition: 'top center' },
      ...(caso.rotuloP ? [{ text: 'P', position: p.nudge(-0.38, 0.3), textposition: 'top center' }] : []),
    ]

    return [
      ...regioes(caso.P).map((pontos, i) => linha2d(
        [...pontos, pontos[0]].map((v) => v.add(offset)),
        { fill: 'toself', fillcolor: alfa(cores.serie[i], 0.25), mode: 'none' }
      )),
      linha2d([a, b, c, a], { line: { color: cores.fraca, width: 1.4 } }),
      ...[a, b, c].map((vertice) =>
        linha2d([vertice, p], { line: { color: cores.fraca, width: 1 } })
      ),
      pontos2d([a, b, c, p], cores.tinta, 8),
      rotulos2d(rotulos, cores.tinta, 14),
      rotulos2d(
        [{ text: caso.titulo, position: vec2(5, 8.2).add(offset), textposition: 'top center' }],
        cores.tinta,
      ),
    ]
  }

  return {
    data: casos.flatMap(painel),
    layout: cena2d([0.8, 26.5], [0.8, 10.0]),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="250" />
</template>
