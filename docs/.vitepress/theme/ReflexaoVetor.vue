<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, vec2, xy } from './geometry.js'

const { isDark } = useData()

const ANGULO = 40 * Math.PI / 180

const X = vec2(0, 0)
const N = vec2(0, 1)
const D = vec2(Math.sin(ANGULO), -Math.cos(ANGULO))

const D_N = D.dot(N)
const PASSO = N.mul(-D_N)
const R = D.add(PASSO.mul(2))

const fig = computed(() => {
  const cores = palette(isDark.value)

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const rotulos = [
    { text: 'd', position: X.sub(D.mul(0.55)).nudged_right(-0.1), textposition: 'middle left' },
    { text: 'n', position: X.add(N.mul(1.08)), textposition: 'top center' },
    { text: 'r', position: R.mul(0.55).nudged_right(-0.08), textposition: 'top left' },
    { text: `d · n = ${D_N.toFixed(2)}`, position: vec2(-0.06, D.y / 2), textposition: 'middle left' },
    { text: '-(d · n) n', position: vec2(D.x, D.y / 2).nudged_right(0.1), textposition: 'middle right' },
    { text: '-(d · n) n', position: vec2(D.x, D.add(PASSO).y + PASSO.y / 2).nudged_right(0.1), textposition: 'middle right' },
  ]

  return {
    data: [
      // a superfície
      linha([vec2(-1.25, 0), vec2(1.45, 0)], { line: { color: cores.tinta, width: 2 } }),
      // a cópia de d saindo de X, que é onde a soma começa
      linha([X, D], { line: { color: cores.serie[0], width: 1.4, dash: 'dot' } }),
      // a decomposição de d: (d · n) n na vertical e o resto na horizontal
      linha([X, vec2(0, D.y)], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      linha([vec2(0, D.y), D], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      {
        ...xy([X, D, R]),
        type: 'scatter', mode: 'markers',
        marker: { size: 8, color: cores.tinta },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy(rotulos.map((r) => r.position)),
        text: rotulos.map((r) => r.text),
        textposition: rotulos.map((r) => r.textposition),
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 13 },
        hoverinfo: 'skip', showlegend: false,
      },
    ],
    layout: {
      ...eixosIguais([-1.35, 1.95], [-1.15, 1.35]),
      annotations: [
        seta(X.sub(D), X, cores.serie[0]),
        seta(X, N, cores.fraca),
        seta(D, D.add(PASSO), cores.serie[1]),
        seta(D.add(PASSO), R, cores.serie[1]),
        seta(X, R, cores.serie[2]),
      ],
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
</template>
