<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, alfa, vec2 } from './geometry.js'

const { isDark } = useData()

const p0 = vec2(0, 0), p1 = vec2(1, 0), p2 = vec2(0, 1)

const rotulos = [
  { text: 'p0', position: p0.nudge(-0.06, -0.06), textposition: 'bottom left' },
  { text: 'p1', position: p1.nudge(0.06, 0.06), textposition: 'top right' },
  { text: 'p2', position: p2.nudge(0.06, 0.06), textposition: 'top right' },
  { text: 'u', position: vec2(1.58, 0), textposition: 'middle right' },
  { text: 'v', position: vec2(0, 1.58), textposition: 'top center' },
  { text: '1', position: p1.nudge(0, -0.08), textposition: 'bottom center' },
  { text: '1', position: p2.nudge(-0.08, 0), textposition: 'middle left' },
  { text: 'v = 0', position: vec2(0.5, 0.0).nudge(0, -0.08), textposition: 'bottom center' },
  { text: 'u = 0', position: vec2(0.0, 0.5).nudge(-0.08, 0), textposition: 'middle left' },
  { text: 'u + v = 1', position: vec2(0.5, 0.5).nudge(0.35, 0), textposition: 'middle center' },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  return {
    data: [
      linha2d([p0, p1, p2, p0], {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.18), mode: 'none',
      }),
      // a reta u + v = 1
      linha2d([vec2(1.5, -0.5), vec2(-0.5, 1.5)], {
        line: { color: cores.serie[0], width: 1.4, dash: 'dash' },
      }),
      pontos2d([p0, p1, p2], cores.tinta),
      rotulos2d(rotulos, cores.tinta, 14),
    ],
    layout: cena2d([-0.85, 1.95], [-0.85, 1.95], {
      annotations: [
        seta2d(vec2(-0.5, 0), vec2(1.5, 0), cores.fraca),
        seta2d(vec2(0, -0.5), vec2(0, 1.5), cores.fraca),
      ],
    }),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
</template>
