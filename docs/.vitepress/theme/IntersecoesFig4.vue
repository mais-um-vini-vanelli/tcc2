<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, alfa, vec2, xy } from './geometry.js'

const { isDark } = useData()

const p0 = vec2(0, 0), p1 = vec2(1, 0), p2 = vec2(0, 1)

const rotulos = [
  { text: 'p0', position: p0.nudged_right(-0.06).nudged_up(-0.06), textposition: 'bottom left' },
  { text: 'p1', position: p1.nudged_right(0.06).nudged_up(0.06), textposition: 'top right' },
  { text: 'p2', position: p2.nudged_right(0.06).nudged_up(0.06), textposition: 'top right' },
  { text: 'u', position: vec2(1.58, 0), textposition: 'middle right' },
  { text: 'v', position: vec2(0, 1.58), textposition: 'top center' },
  { text: '1', position: p1.nudged_up(-0.08), textposition: 'bottom center' },
  { text: '1', position: p2.nudged_right(-0.08), textposition: 'middle left' },
  { text: 'v = 0', position: vec2(0.5, 0.0).nudged_up(-0.08), textposition: 'bottom center' },
  { text: 'u = 0', position: vec2(0.0, 0.5).nudged_right(-0.08), textposition: 'middle left' },
  { text: 'u + v = 1', position: vec2(0.5, 0.5).nudged_right(0.35), textposition: 'middle center' },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  return {
    data: [
      linha([p0, p1, p2, p0], {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.18), mode: 'none',
      }),
      // a reta u + v = 1
      linha([vec2(1.5, -0.5), vec2(-0.5, 1.5)], {
        line: { color: cores.serie[0], width: 1.4, dash: 'dash' },
      }),
      {
        ...xy([p0, p1, p2]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cores.tinta },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy(rotulos.map((r) => r.position)),
        text: rotulos.map((r) => r.text),
        textposition: rotulos.map((r) => r.textposition),
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 14 },
        hoverinfo: 'skip', showlegend: false,
      },
    ],
    layout: {
      ...eixosIguais([-0.85, 1.95], [-0.85, 1.95]),
      annotations: [
        seta(vec2(-0.5, 0), vec2(1.5, 0), cores.fraca),
        seta(vec2(0, -0.5), vec2(0, 1.5), cores.fraca),
      ],
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
</template>
