<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, malha3d, pontos3d, rotulos3d, alfa, vec3 } from './geometry.js'

const { isDark } = useData()

const P0 = vec3(0, 0.55, 0)
const P1 = vec3(1.7, 0.55, 0.35)
const P2 = vec3(0.5, 1.6, -0.6)
const U = 0.3
const V = 0.45

const E1 = P1.sub(P0)
const E2 = P2.sub(P0)
const P = P0.add(E1.mul(U)).add(E2.mul(V))

const paraFora = (ponto, quanto = 0.2) => {
  const CENTROIDE = P0.add(E1.mul(1 / 3)).add(E2.mul(1 / 3))

  return ponto.add(ponto.sub(CENTROIDE).normalized().mul(quanto))
}

const ALCANCE = { x: [-0.30, 2.00], y: [-0.30, 2.15], z: [-0.90, 0.65] }

const fig = computed(() => {
  const cores = palette(isDark.value)

  const emE1 = P0.add(E1.mul(U))
  const emE2 = P0.add(E2.mul(V))
  const noChao = (p) => vec3(p.x, 0, p.z)

  const rotulos = [
    { text: 'p0', position: paraFora(P0) },
    { text: 'p1', position: paraFora(P1) },
    { text: 'p2', position: paraFora(P2) },
    { text: 'E1', position: paraFora(P0.add(E1.mul(0.4))) },
    { text: 'E2', position: paraFora(P0.add(E2.mul(0.5))) },
    { text: '(u · E1, v · E2)', position: P.add(vec3(0.1, 0.1, 0)), textposition: 'middle right' },
  ]

  return {
    data: [
      linha3d([P0, P1, P2].flatMap((p) => [p, noChao(p), null]), {
        line: { color: cores.fraca, width: 1, dash: 'dot' },
      }),
      linha3d([P0, P1, P2, P0].map(noChao), { line: { color: cores.fraca, width: 1 } }),
      malha3d([P0, P1, P2], { i: [0], j: [1], k: [2] }, {
        color: cores.serie[0], opacity: 0.16, flatshading: true,
      }),
      linha3d([P0, P1, P2, P0], { line: { color: alfa(cores.serie[0], 0.8), width: 2 } }),
      linha3d([emE1, P, emE2], { line: { color: cores.fraca, width: 1.4, dash: 'dot' } }),
      ...seta3d(P0, P1, cores.serie[1]),
      ...seta3d(P0, P2, cores.serie[2]),
      pontos3d([P0, P1, P2, emE1, emE2], cores.tinta, 4),
      pontos3d([P], cores.tinta, 7),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.5, 1.1, 1.8), ALCANCE),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="380" />
</template>
