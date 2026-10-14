<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, malha3d, pontos3d, rotulos3d, paralelepipedo, alfa, vec3 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const angulo = ref(55)

const GRAU = Math.PI / 180

const P0 = vec3(0, 0, 0)
const P1 = vec3(1.7, 0, 0.35)
const P2 = vec3(0.55, 0, -1.2)

const E1 = P1.sub(P0)
const E2 = P2.sub(P0)

const BISSETRIZ = E1.normalized().add(E2.normalized()).normalized()
const NORMAL = E1.cross(E2).normalized()

const D = computed(() => BISSETRIZ.mul(Math.cos(angulo.value * GRAU)).add(NORMAL.mul(Math.sin(angulo.value * GRAU))))
const DET = computed(() => E1.dot(D.value.cross(E2)))

const ALCANCE = { x: [-0.30, 3.25], y: [-0.30, 1.25], z: [-1.85, 0.65] }

const fig = computed(() => {
  const cores = palette(isDark.value)

  const caixa = paralelepipedo(P0, E1, E2, D.value)

  const rotulos = [
    { text: 'p0', position: P0, textposition: 'middle left' },
    { text: 'E1', position: P0.add(E1.mul(0.5)).add(vec3(0, -0.1, 0)), textposition: 'bottom center' },
    { text: 'E2', position: P0.add(E2.mul(0.5)).add(vec3(0, -0.1, 0)), textposition: 'bottom center' },
    { text: 'D', position: P0.add(D.value.mul(0.6)).add(vec3(-0.10, 0.05, 0.07)), textposition: 'middle left' },
  ]

  return {
    data: [
      malha3d(caixa.cantos, caixa.faces, {
        color: cores.serie[0], opacity: 0.13, flatshading: true,
      }),
      linha3d(caixa.arestas, { line: { color: cores.fraca, width: 1.2 } }),
      ...seta3d(P0, P1, cores.serie[1]),
      ...seta3d(P0, P2, cores.serie[2]),
      ...seta3d(P0, P0.add(D.value), cores.serie[0]),
      pontos3d([P0], cores.tinta),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.0, 0.75, 1.4), ALCANCE, 2.0),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Slider v-model="angulo" label="D" :min="0" :max="90" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
    <p class="legenda">det(M) = E1 · (D × E2) = {{ DET.toFixed(2) }}</p>
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
