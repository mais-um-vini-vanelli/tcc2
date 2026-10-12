<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, alfa, arco, vec2, xy } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const theta = ref(55)

const W = 1.3
const A = vec2(0, 0)
const COMPRIMENTO = 2.8

const fig = computed(() => {
  const cores = palette(isDark.value)

  const theta_rad = theta.value * Math.PI / 180
  const L = vec2(-Math.sin(theta_rad), Math.cos(theta_rad))
  const S = W / Math.cos(theta_rad)
  const B = vec2(S, 0)
  const C = B.add(L.mul(A.sub(B).dot(L)))

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const CA = A.sub(C).normalized()
  const CB = B.sub(C).normalized()
  const AC = C.sub(A).normalized()

  const rotulos = [
    { text: 'A', position: A.nudged_up(-0.1), textposition: 'bottom center' },
    { text: 'B', position: B.nudged_up(-0.1), textposition: 'bottom center' },
    { text: 'C', position: C.add(AC.mul(0.16)), textposition: 'middle center' },
    { text: `s = w / cos θ = ${S.toFixed(2)}`, position: vec2(S / 2, -0.12), textposition: 'bottom center' },
    { text: `w = ${W}`, position: C.add(A).mul(0.5).nudged_right(-0.1), textposition: 'middle left' },
    { text: 'n', position: B.nudged_up(1.12), textposition: 'top center' },
    { text: 'l', position: B.add(L).nudged_up(0.12), textposition: 'top center' },
    { text: `θ`, position: B.add(vec2(1, 0).rotated(Math.PI / 2 + theta_rad / 2).mul(0.55)), textposition: 'middle center' },
    { text: `90° − θ`, position: B.add(vec2(1, 0).rotated(Math.PI / 2 + theta_rad + (90 - theta_rad) / 2).mul(0.55)), textposition: 'top left' },
  ]

  return {
    data: [
      // a área azul
      linha([A, B, B.add(L.mul(COMPRIMENTO)), A.add(L.mul(COMPRIMENTO)), A], {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.14), mode: 'none',
      }),
      ...[A, B].map((p) => linha([p, p.add(L.mul(COMPRIMENTO))], {
        line: { color: cores.serie[0], width: 1.2 },
      })),
      // a reta da superfície
      linha([vec2(-2.5, 0), vec2(4.3, 0)], { line: { color: cores.tinta, width: 2 } }),
      // o triângulo retângulo ACB
      linha([A, C, B], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      // o ângulo reto
      linha([C.add(CA.mul(0.16)), C.add(CA.mul(0.16)).add(CB.mul(0.16)), C.add(CB.mul(0.16))], {
        line: { color: cores.fraca, width: 1.2 },
      }),
      // os dois ângulos em B
      linha(arco(B, 0.4, Math.PI / 2, Math.PI / 2 + theta_rad), { line: { color: cores.fraca, width: 1.2 } }),
      linha(arco(B, 0.4, Math.PI, Math.PI / 2 + theta_rad), { line: { color: cores.serie[2], width: 1.2 } }),
      {
        ...xy([A, B, C]),
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
      ...eixosIguais([-2.7, 4.6], [-0.85, 2.75]),
      annotations: [
        seta(A, C, cores.serie[1]),
        seta(B, B.nudged_up(1.0), cores.fraca),
        seta(B, B.add(L.mul(1.0)), cores.serie[2]),
      ],
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Slider v-model="theta" label="θ" :min="5" :max="70" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="370" />
</template>
