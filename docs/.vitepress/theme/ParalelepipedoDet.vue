<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, malha3d, pontos3d, rotulos3d, paralelepipedo, vec3 } from './geometry.js'

const { isDark } = useData()

const ORIGEM = vec3(0, 0, 0)
const A = vec3(1.5, 0, 0.3)
const B = vec3(0.3, 1.3, 0.2)
const C = vec3(0.4, 0.2, 1.4)

const DET = A.dot(B.cross(C))
const CAIXA = paralelepipedo(ORIGEM, A, B, C)

const ALCANCE = { x: [-0.25, 2.45], y: [-0.25, 1.75], z: [-0.25, 2.15] }

const fig = computed(() => {
  const cores = palette(isDark.value)

  const rotulos = [
    { text: 'a', position: A.mul(0.7).add(vec3(0, -0.16, 0)), textposition: 'middle center' },
    { text: 'b', position: B.mul(0.7).add(vec3(-0.14, 0, 0)), textposition: 'middle left' },
    { text: 'c', position: C.mul(0.7).add(vec3(-0.12, -0.12, 0)), textposition: 'middle left' },
  ]

  return {
    data: [
      malha3d(CAIXA.cantos, CAIXA.faces, {
        color: cores.serie[0], opacity: 0.14, flatshading: true,
      }),
      linha3d(CAIXA.arestas, { line: { color: cores.fraca, width: 1.2 } }),
      ...seta3d(ORIGEM, A, cores.serie[0]),
      ...seta3d(ORIGEM, B, cores.serie[1]),
      ...seta3d(ORIGEM, C, cores.serie[2]),
      pontos3d([ORIGEM], cores.tinta),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(-0.5, 0.5, 1.5), ALCANCE),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="380" />
    <p class="legenda">volume = det(a, b, c) = {{ DET.toFixed(2) }}</p>
</template>

<style scoped>
.legenda {
    margin: -1rem 0 1.5rem;
    text-align: center;
    font-family: var(--vp-font-family-mono);
    font-size: 0.85rem;
    color: var(--vp-c-text-2);
}
</style>

