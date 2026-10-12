<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette } from './geometry.js'

const { isDark } = useData()

const GAMA = 2.2
const BYTE = 255
const PASSOS = 80
const C = Array.from({ length: PASSOS + 1 }, (_, i) => i / PASSOS)
const MEIO = 0.5

const semGama = (c) => c * BYTE
const comGama = (c) => Math.pow(c, 1 / GAMA) * BYTE

const fig = computed(() => {
  const cores = palette(isDark.value)

  const curva = (f, extra) => ({
    x: C, y: C.map(f), type: 'scatter', mode: 'lines',
    hovertemplate: 'linear %{x:.2f} → sRGB %{y:.0f}<extra></extra>', ...extra,
  })

  const marcados = [
    { y: semGama(MEIO), textposition: 'bottom right' },
    { y: comGama(MEIO), textposition: 'top left' },
  ]

  return {
    data: [
      curva(semGama, {
        name: 'sem gama', line: { color: cores.fraca, width: 1.4, dash: 'dash' },
      }),
      curva(comGama, {
        name: 'com gama', line: { color: cores.serie[0], width: 2.5 },
      }),
      {
        x: marcados.map(() => MEIO),
        y: marcados.map((m) => m.y),
        text: marcados.map((m) => `byte = ${Math.round(m.y)} `),
        textposition: marcados.map((m) => m.textposition),
        type: 'scatter', mode: 'markers+text',
        marker: { size: 8, color: cores.tinta },
        textfont: { color: cores.tinta, size: 12 },
        hoverinfo: 'skip', showlegend: false,
      },
    ],
    layout: {
      xaxis: {
        title: { text: 'entrada (linear, f32)', font: { size: 12 } },
        range: [0, 1], dtick: 0.25, fixedrange: true, constrain: 'domain', zeroline: false,
      },
      yaxis: {
        title: { text: 'saída (sRGB, u8)', font: { size: 12 } },
        range: [0, BYTE], tickvals: [0, 64, 128, 192, BYTE],
        fixedrange: true, constrain: 'domain', zeroline: false,
        scaleanchor: 'x', scaleratio: 1 / BYTE,
      },
      shapes: [
        { type: 'line', x0: MEIO, x1: MEIO, y0: 0, y1: comGama(MEIO),
          line: { color: cores.fraca, width: 1, dash: 'dot' } },
      ],
      legend: {
        orientation: 'h', x: 0.5, xanchor: 'center', y: -0.18, yanchor: 'top',
        font: { color: cores.tinta, size: 12 },
      },
      margin: { t: 10, r: 10, b: 70, l: 58 },
      hovermode: 'closest',
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
</template>
