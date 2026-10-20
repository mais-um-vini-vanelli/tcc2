<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, arco2d, alfa, vec2 } from './geometry.js'
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

  const CA = A.sub(C).normalized()
  const CB = B.sub(C).normalized()
  const AC = C.sub(A).normalized()

  const rotulos = [
    { text: 'A', position: A.nudge(0, -0.1), textposition: 'bottom center' },
    { text: 'B', position: B.nudge(0, -0.1), textposition: 'bottom center' },
    { text: 'C', position: C.add(AC.mul(0.16)), textposition: 'middle center' },
    { text: `s = w / cos θ = ${S.toFixed(2)}`, position: vec2(S / 2, -0.12), textposition: 'bottom center' },
    { text: `w = ${W}`, position: C.add(A).mul(0.5).nudge(-0.1, 0), textposition: 'middle left' },
    { text: 'n', position: B.nudge(0, 1.12), textposition: 'top center' },
    { text: 'l', position: B.add(L).nudge(0, 0.12), textposition: 'top center' },
    { text: `θ`, position: B.add(vec2(1, 0).rotated(Math.PI / 2 + theta_rad / 2).mul(0.55)), textposition: 'middle center' },
    { text: `90° − θ`, position: B.add(vec2(1, 0).rotated(Math.PI / 2 + theta_rad + (Math.PI / 2 - theta_rad) / 2).mul(0.55)), textposition: 'top left' },
  ]

  return {
    data: [
      // a área azul
      linha2d([A, B, B.add(L.mul(COMPRIMENTO)), A.add(L.mul(COMPRIMENTO)), A], {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.14), mode: 'none',
      }),
      ...[A, B].map((p) => linha2d([p, p.add(L.mul(COMPRIMENTO))], {
        line: { color: cores.serie[0], width: 1.2 },
      })),
      // a reta da superfície
      linha2d([vec2(-2.5, 0), vec2(4.3, 0)], { line: { color: cores.tinta, width: 2 } }),
      // o triângulo retângulo ACB
      linha2d([A, C, B], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      // o ângulo reto
      linha2d([C.add(CA.mul(0.16)), C.add(CA.mul(0.16)).add(CB.mul(0.16)), C.add(CB.mul(0.16))], {
        line: { color: cores.fraca, width: 1.2 },
      }),
      // os dois ângulos em B
      linha2d(arco2d(B, 0.4, Math.PI / 2, Math.PI / 2 + theta_rad), { line: { color: cores.fraca, width: 1.2 } }),
      linha2d(arco2d(B, 0.4, Math.PI, Math.PI / 2 + theta_rad), { line: { color: cores.serie[2], width: 1.2 } }),
      pontos2d([A, B, C], cores.tinta),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-2.7, 4.6], [-0.85, 2.75], {
      annotations: [
        seta2d(A, C, cores.serie[1]),
        seta2d(B, B.nudge(0, 1.0), cores.fraca),
        seta2d(B, B.add(L.mul(1.0)), cores.serie[2]),
      ],
    }),
  }
})
</script>

<template>
    <Slider v-model="theta" label="θ" :min="5" :max="70" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="370" />
</template>
