<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, vec2 } from './geometry.js'

const { isDark } = useData()

const INCLINACAO = 15 * Math.PI / 180
const P0 = vec2(0, 0)
const U = vec2(1, 0).rotated(INCLINACAO)
const N = vec2(0, 1).rotated(INCLINACAO)

const PONTOS = [
  { nome: 'p', u: 1.5, distancia: 1.1 },
  { nome: "p'", u: 0.3, distancia: -0.9 },
]

const posicaoV = ({ u, distancia }) => P0.add(U.mul(u)).add(N.mul(distancia))
const noPlano = ({ u }) => P0.add(U.mul(u))

const fig = computed(() => {
  const cores = palette(isDark.value)

  const rotulos = [
    { text: 'p0', position: P0.add(N.mul(-0.12)).add(U.mul(-0.12)), textposition: 'bottom center' },
    { text: 'n', position: P0.add(N.mul(1.55)), textposition: 'top center' },
    { text: 'plano', position: P0.add(U.mul(-1.9)).add(N.mul(0.12)), textposition: 'top center' },
    ...PONTOS.map((ponto) => ({
      text: ponto.nome,
      position: posicaoV(ponto).nudge(0.12, 0),
      textposition: 'middle right',
    })),
    ...PONTOS.map((ponto) => ({
      text: `n · v = ${ponto.distancia > 0 ? '+' : ''}${ponto.distancia}`,
      position: noPlano(ponto).add(N.mul(ponto.distancia / 2)).add(U.mul(0.12)),
      textposition: 'middle right',
    })),
    ...PONTOS.map((ponto, i) => ({
      text: i === 0 ? 'v' : "v'",
      position: posicaoV(ponto).label(P0, i === 0 ? 0.12 : -0.14),
      textposition: 'middle center',
    })),
  ]

  return {
    data: [
      // a reta do plano
      linha2d([P0.add(U.mul(-2.2)), P0.add(U.mul(2.6))], {
        line: { color: cores.tinta, width: 2 },
      }),
      // as linhas pontilhadas horizontais
      ...PONTOS.map((ponto, i) => linha2d([noPlano(ponto), posicaoV(ponto)], {
        line: { color: cores.serie[i], width: 1.4, dash: 'dot' },
      })),
      pontos2d([P0, ...PONTOS.map(posicaoV)], cores.tinta),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-2.5, 3.0], [-1.6, 2.0], {
      annotations: [
        seta2d(P0, P0.add(N.mul(1.5)), cores.tinta),
        ...PONTOS.map((ponto, i) => seta2d(P0, posicaoV(ponto), cores.serie[i])),
      ],
    }),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
</template>
