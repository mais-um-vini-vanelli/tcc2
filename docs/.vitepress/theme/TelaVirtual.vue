<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, malha3d, pontos3d, rotulos3d, arco3d, alfa, vec3 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const fov = ref(60)

const GRAU = Math.PI / 180
const ASPECT = 16 / 9
const CAMERA = vec3(0, 0, 0)
const FRENTE = vec3(0, 0, 1)
const CIMA = vec3(0, 1, 0)

const ALCANCE = { x: [-1.93, 1.93], y: [-1.15, 1.15], z: [-0.15, 1.15] }

const meiaAltura = computed(() => Math.tan(fov.value / 2 * GRAU))

const fig = computed(() => {
  const cores = palette(isDark.value)

  const meiaLargura = meiaAltura.value * ASPECT

  const centro = vec3(0, 0, 1)
  const topo = centro.add(vec3(0, meiaAltura.value, 0))
  const base = centro.sub(vec3(0, meiaAltura.value, 0))

  const cantos = [
    vec3(-meiaLargura, -meiaAltura.value, 1), 
    vec3( meiaLargura, -meiaAltura.value, 1),
    vec3( meiaLargura,  meiaAltura.value, 1), 
    vec3(-meiaLargura,  meiaAltura.value, 1),
  ]

  const rotulos = [
    { text: 'câmera', position: CAMERA.add(vec3(0, -0.18, 0)) },
    { text: 'z = 1', position: centro.add(vec3(0, -0.18, 0)) },
    { text: 'fov/2', position: arco3d(CAMERA, 0.88, FRENTE, CIMA, 0, fov.value / 4 * GRAU).at(-1) },
  ]

  return {
    data: [
      malha3d(cantos, { i: [0, 0], j: [1, 2], k: [2, 3] }, {
        color: cores.serie[0], opacity: 0.14, flatshading: true,
      }),
      linha3d([...cantos, cantos[0]], { line: { color: alfa(cores.serie[0], 0.85), width: 2 } }),
      linha3d(cantos.flatMap((canto) => [CAMERA, canto, null]), {
        line: { color: cores.fraca, width: 1 },
      }),
      linha3d([centro, topo], { line: { color: cores.serie[1], width: 2 } }),
      linha3d([CAMERA, topo], { line: { color: cores.tinta, width: 1.4 } }),
      linha3d([CAMERA, base], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      linha3d(arco3d(CAMERA, 0.5, FRENTE, CIMA, 0, fov.value / 2 * GRAU), {
        line: { color: cores.fraca, width: 1.4 },
      }),
      ...seta3d(CAMERA, centro, cores.serie[2]),
      pontos3d([CAMERA], cores.tinta, 6),
      pontos3d([centro, topo], cores.tinta, 4),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(-1.8, 0.7, -1.3), ALCANCE, 2.15),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Slider v-model="fov" label="fov" :min="30" :max="90" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
    <p class="legenda">meia_altura = tan(fov/2) = {{ meiaAltura.toFixed(2) }}</p>
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

