<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, alfa, vec2, xy } from './geometry.js'

const { isDark } = useData()

const A = vec2(2.5, 2), B = vec2(7.5, 4), C = vec2(4, 6.5)

// os subtriângulos de P, na mesma ordem de TrianguloFig2
const regioes = (P) => [[B, C, P], [C, A, P], [A, B, P]]

const casos = [
  {
    titulo: 'P em um vértice',
    P: A,
    pesosBC: ["0", "0"],
    rotuloA: (a) => [a.nudged_up(-0.30), 'A = P (1)', 'bottom center'],
    rotuloP: false,
  },
  {
    titulo: 'P no meio de AB',
    P: vec2(5, 3), 
    pesosBC: ["½", "0"],
    rotuloA: (a) => [a.nudged_right(-0.42), 'A (½)', 'middle left'],
    rotuloP: true,
  },
  {
    titulo: 'P no centro',
    P: vec2(14 / 3, 13 / 3), 
    pesosBC: ["⅓", "⅓"],
    rotuloA: (a) => [a.nudged_right(-0.42), 'A (⅓)', 'middle left'],
    rotuloP: true,
  },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const painel = (caso, index) => {
    const offset = vec2(index * 8.6, 0)
    const [a, b, c, p] = [A, B, C, caso.P].map((v) => v.add(offset))

    const rotulos = [
      caso.rotuloA(a),
      [b.nudged_right(0.42), `B (${caso.pesosBC[0]})`, 'middle right'],
      [c.nudged_up(0.3), `C (${caso.pesosBC[1]})`, 'top center'],
      ...(caso.rotuloP ? [[p.nudged_right(-0.38).nudged_up(0.3), 'P', 'top center']] : []),
    ]

    return [
      ...regioes(caso.P).map((pontos, i) => linha(
        [...pontos, pontos[0]].map((v) => v.add(offset)),
        { fill: 'toself', fillcolor: alfa(cores.serie[i], 0.25), mode: 'none' }
      )),
      linha([a, b, c, a], { line: { color: cores.fraca, width: 1.4 } }),
      ...[a, b, c].map((vertice) =>
        linha([vertice, p], { line: { color: cores.fraca, width: 1 } })
      ),
      {
        ...xy([a, b, c, p]),
        type: 'scatter', mode: 'markers',
        marker: { size: 8, color: cores.tinta },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy(rotulos.map((r) => r[0])),
        text: rotulos.map((r) => r[1]),
        textposition: rotulos.map((r) => r[2]),
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 14 },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy([vec2(5, 8.2).add(offset)]),
        text: [caso.titulo],
        type: 'scatter', mode: 'text', textposition: 'top center',
        textfont: { color: cores.tinta, size: 13 },
        hoverinfo: 'skip', showlegend: false,
      },
    ]
  }

  return {
    data: casos.flatMap(painel),
    layout: {
      ...eixosIguais([0.8, 26.5], [0.8, 10.0]),
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="250" />
</template>
