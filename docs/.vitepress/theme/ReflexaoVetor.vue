<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, vec2 } from './geometry.js'

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

  const rotulos = [
    { text: 'd', position: X.sub(D.mul(0.55)).nudge(-0.1, 0), textposition: 'middle left' },
    { text: 'n', position: X.add(N.mul(1.08)), textposition: 'top center' },
    { text: 'r', position: R.mul(0.55).nudge(-0.08, 0), textposition: 'top left' },
    { text: `d · n = ${D_N.toFixed(2)}`, position: vec2(-0.06, D.y / 2), textposition: 'middle left' },
    { text: '-(d · n) n', position: vec2(D.x, D.y / 2).nudge(0.1, 0), textposition: 'middle right' },
    { text: '-(d · n) n', position: vec2(D.x, D.add(PASSO).y + PASSO.y / 2).nudge(0.1, 0), textposition: 'middle right' },
  ]

  return {
    data: [
      // a superfície
      linha2d([vec2(-1.25, 0), vec2(1.45, 0)], { line: { color: cores.tinta, width: 2 } }),
      // a cópia de d saindo de X, que é onde a soma começa
      linha2d([X, D], { line: { color: cores.serie[0], width: 1.4, dash: 'dot' } }),
      // a decomposição de d: (d · n) n na vertical e o resto na horizontal
      linha2d([X, vec2(0, D.y)], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      linha2d([vec2(0, D.y), D], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      pontos2d([X, D, R], cores.tinta, 8),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-1.35, 1.95], [-1.15, 1.35], {
      annotations: [
        seta2d(X.sub(D), X, cores.serie[0]),
        seta2d(X, N, cores.fraca),
        seta2d(D, D.add(PASSO), cores.serie[1]),
        seta2d(D.add(PASSO), R, cores.serie[1]),
        seta2d(X, R, cores.serie[2]),
      ],
    }),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
</template>
