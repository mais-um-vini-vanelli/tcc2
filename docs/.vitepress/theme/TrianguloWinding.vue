<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, malha3d, rotulos3d, vec3 } from './geometry.js'

const { isDark } = useData()

const A = vec3(-1.05, -0.7, 0)
const B = vec3(1.1, -0.55, 0)
const C = vec3(0.05, 1, 0)

const CENTRO = A.add(B).add(C).mul(1 / 3)
const NORMAL = B.sub(A).cross(C.sub(A)).normalized()
const PONTA_NORMAL = CENTRO.add(NORMAL.mul(0.8))

const fig = computed(() => {
  const cores = palette(isDark.value)

  const rotulos = [A, B, C].map((p, i) => ({
    text: 'ABC'[i],
    position: p.add(p.sub(CENTRO).normalized().mul(0.26)),
  }))

  return {
    data: [
      malha3d([A, B, C], { i: [0], j: [1], k: [2] }, {
        color: cores.serie[0], opacity: 0.12, flatshading: true,
      }),
      ...seta3d(CENTRO, PONTA_NORMAL, cores.fraca, 0.13),
      ...seta3d(A, B, cores.serie[0]),
      ...seta3d(B, C, cores.serie[0]),
      ...seta3d(C, A, cores.serie[0]),
      rotulos3d([...rotulos, { text: 'n', position: PONTA_NORMAL.add(NORMAL.mul(0.16)) }], cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.2, 0.7, 1.8), { x: [-1.5, 1.5], y: [-1.15, 1.5], z: [-0.35, 1.15] }, 1.55),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
    <p class="legenda">arraste pra girar</p>
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
