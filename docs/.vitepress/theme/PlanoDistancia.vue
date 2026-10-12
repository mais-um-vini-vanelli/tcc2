<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, vec2, xy } from './geometry.js'

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

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const rotulos = [
    { text: 'p0', position: P0.add(N.mul(-0.12)).add(U.mul(-0.12)), textposition: 'bottom center' },
    { text: 'n', position: P0.add(N.mul(1.55)), textposition: 'top center' },
    { text: 'plano', position: P0.add(U.mul(-1.9)).add(N.mul(0.12)), textposition: 'top center' },
    ...PONTOS.map((ponto) => ({
      text: ponto.nome,
      position: posicaoV(ponto).nudged_right(0.12),
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
      linha([P0.add(U.mul(-2.2)), P0.add(U.mul(2.6))], {
        line: { color: cores.tinta, width: 2 },
      }),
      // as linhas pontilhadas horizontais
      ...PONTOS.map((ponto, i) => linha([noPlano(ponto), posicaoV(ponto)], {
        line: { color: cores.serie[i], width: 1.4, dash: 'dot' },
      })),
      {
        ...xy([P0, ...PONTOS.map(posicaoV)]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cores.tinta },
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
      ...eixosIguais([-2.5, 3.0], [-1.6, 2.0]),
      annotations: [
        seta(P0, P0.add(N.mul(1.5)), cores.tinta),
        ...PONTOS.map((ponto, i) => seta(P0, posicaoV(ponto), cores.serie[i])),
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
