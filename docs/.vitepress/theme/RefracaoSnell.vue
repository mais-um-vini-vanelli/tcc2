<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, arco2d, vec2 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const theta1 = ref(40)
const eta = ref(0.67)

const X = vec2(0, 0)
const N = vec2(0, 1)

const fig = computed(() => {
  const cores = palette(isDark.value)

  const theta1_rad = theta1.value * Math.PI / 180

  const senTheta2 = eta.value * Math.sin(theta1_rad)
  const total = senTheta2 > 1
  const theta2 = total ? 0 : Math.asin(senTheta2)

  const menosD = vec2(-Math.sin(theta1_rad), Math.cos(theta1_rad))
  const T = vec2(Math.sin(theta2), -Math.cos(theta2))

  const rotulos = [
    { text: '−d', position: menosD.mul(1.06), textposition: 'top center' },
    { text: 'n', position: N.mul(1.08), textposition: 'top center' },
    { text: 'u', position: vec2(1.32, 0), textposition: 'middle right' },
    { text: `θ₁ = ${theta1.value}°`, position: vec2(0, 0.62).rotated(-theta1_rad / 2), textposition: 'middle left' },
    ...(total
      ? [{ text: 'reflexão interna total', position: vec2(0, -0.75), textposition: 'middle center' }]
      : [
          { text: 't', position: T.mul(1.06), textposition: 'bottom center' },
          {
            text: `θ₂ = ${(theta2 * 180 / Math.PI).toFixed(1)}°`,
            position: vec2(0, -0.62).rotated(theta2 / 2),
            textposition: 'middle right',
          },
        ]),
  ]

  return {
    data: [
      // as retas da superfície e da normal
      linha2d([vec2(-1.3, 0), vec2(1.25, 0)], { line: { color: cores.tinta, width: 2 } }),
      linha2d([vec2(0, -1.25), vec2(0, 1.25)], { line: { color: cores.fraca, width: 1, dash: 'dot' } }),
      // os ângulos
      linha2d(arco2d(X, 0.5, Math.PI / 2, Math.PI / 2 + theta1_rad), { line: { color: cores.fraca, width: 1.2 } }),
      ...(total
        ? []
        : [linha2d(arco2d(X, 0.5, -Math.PI / 2, -Math.PI / 2 + theta2), {
            line: { color: cores.fraca, width: 1.2 },
          })]),
      pontos2d([X], cores.tinta),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-1.5, 1.6], [-1.45, 1.45], {
      annotations: [
        seta2d(X, menosD, cores.serie[0]),
        seta2d(X, N, cores.fraca),
        seta2d(X, vec2(1.2, 0), cores.fraca),
        ...(total ? [] : [seta2d(X, T, cores.serie[1])]),
      ],
    }),
  }
})
</script>

<template>
    <Slider v-model="theta1" label="θ₁" :min="5" :max="85" sufixo="°" />
    <Slider v-model="eta" label="η = n1/n2" :min="0.4" :max="1.8" :step="0.01" />
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
</template>
