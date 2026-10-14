<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, pontos3d, rotulos3d, arco3d, vec3 } from './geometry.js'

const { isDark } = useData()

const CENTRO = vec3(0, 0, 0)
const RAIO = 1

const X = vec3(1, 0, 0)
const Y = vec3(0, 1, 0)
const Z = vec3(0, 0, 1)

const GRAU = Math.PI / 180
const theta = 55 * GRAU
const phi = 35 * GRAU

const P = CENTRO.add(vec3(
  Math.sin(theta) * Math.cos(phi),
  Math.cos(theta),
  Math.sin(theta) * Math.sin(phi),
).mul(RAIO))

const ALCANCE = { x: [-1.05, 1.05], y: [-1.35, 1.35], z: [-1.05, 1.05] }

const fig = computed(() => {
  const cores = palette(isDark.value)

  const rotulos = [
    { text: 'P', position: P.add(vec3(0, 0.18, 0)) },
    { text: 'P - centro', position: P.sub(CENTRO).mul(0.5) },
    { text: '|P - centro| = raio', position: CENTRO.add(vec3(0, -RAIO - 0.18, 0)), textposition: 'bottom center' },
  ]

  return {
    data: [
      // os paralelos
      ...[30, 60, 90, 120, 150].map((theta_graus) => {
        const t = theta_graus * GRAU

        return linha3d(arco3d(CENTRO.add(Y.mul(Math.cos(t) * RAIO)), Math.sin(t) * RAIO, X, Z), { line: { color: cores.fraca, width: 2 } })
      }),
      // os meridianos
      ...[0, 45, 90, 135].map((phi_graus) => {
        const t = phi_graus * GRAU

        return linha3d(arco3d(CENTRO, RAIO, vec3(Math.cos(t), 0, Math.sin(t)), Y), { line: { color: cores.fraca, width: 2 } })
      }),
      ...seta3d(CENTRO, P, cores.serie[0]),
      pontos3d([CENTRO, P], cores.tinta),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.4, 0.7, 1.9), ALCANCE),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="380" />
</template>
