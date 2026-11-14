<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, pontos3d, rotulos3d, vec3 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const cameraX = ref(-1.1)
const cameraY = ref(1.25)
const cameraZ = ref(1.35)

const mundo = {
  origem: vec3(0, 0, 0),
  eixos: [
    { nome: 'x', direcao: vec3(1, 0, 0) },
    { nome: 'y', direcao: vec3(0, 1, 0) },
    { nome: 'z', direcao: vec3(0, 0, 1) },
  ],
  tamanhoEixos: 1.35,
}

const PONTO = vec3(1.0, 0.9, 0.3)

const FRENTE = vec3(0.6, -1.3, -1.0).normalized()
const DIREITA = FRENTE.cross(mundo.eixos[1].direcao).normalized()
const CIMA = DIREITA.cross(FRENTE)

const camera = {
  eixos: [
    { nome: 'direita', direcao: DIREITA },
    { nome: 'cima', direcao: CIMA },
    { nome: 'frente', direcao: FRENTE },
  ],
  tamanhoEixos: 1.1,
}

const origem = computed(() => vec3(cameraX.value, cameraY.value, cameraZ.value))

const local = computed(() => {
  const d = PONTO.sub(origem.value)

  return vec3(d.dot(DIREITA), d.dot(CIMA), d.dot(FRENTE))
})

const formataVetor = (v) => `(${v.x.toFixed(2)}, ${v.y.toFixed(2)}, ${v.z.toFixed(2)})`

const fig = computed(() => {
  const cores = palette(isDark.value)

  const d = PONTO.sub(origem.value)

  const rotulos = [
    ...mundo.eixos.map(({ nome, direcao }) => ({
      text: nome, position: direcao.mul(mundo.tamanhoEixos + 0.14),
    })),
    ...camera.eixos.map(({ nome, direcao }) => ({
      text: nome, position: origem.value.add(direcao.mul(camera.tamanhoEixos + 0.3)),
    })),
    { text: 'mundo', position: mundo.origem.add(vec3(0, -0.25, 0)) },
    { text: 'câmera', position: origem.value.add(FRENTE.mul(-0.25)) },
    { text: 'ponto', position: PONTO.add(vec3(0, 0.25, 0)) },
    { text: 'd', position: origem.value.add(d.mul(0.5)).add(vec3(0, 0.18, 0)) },
  ]

  return {
    data: [
      ...mundo.eixos.flatMap(({ direcao }, n) => seta3d(mundo.origem, direcao.mul(mundo.tamanhoEixos), cores.serie[n], 0.14)),
      ...camera.eixos.flatMap(({ direcao }, n) => seta3d(origem.value, origem.value.add(direcao.mul(camera.tamanhoEixos)), cores.serie[n], 0.14)),
      ...seta3d(origem.value, PONTO, cores.tinta, 0.14),
      linha3d([mundo.origem, PONTO], { line: { color: cores.fraca, width: 1.4, dash: 'dot' } }),
      pontos3d([mundo.origem, (origem.value), PONTO], cores.tinta, 6),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.6, 1.0, 2.2), { x: [-1.5, 1.6], y: [-0.7, 2.3], z: [-0.8, 2.1] }),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Slider v-model="cameraX" label="câmera x" :min="-1.6" :max="0" :step="0.1" />
    <Slider v-model="cameraY" label="câmera y" :min="0.5" :max="1.4" :step="0.1" />
    <Slider v-model="cameraZ" label="câmera z" :min="0.6" :max="1.4" :step="0.1" />
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
    <div class="coordenadas">
        <span>p mundo</span><span>{{ formataVetor(PONTO) }}</span>
        <span>p câmera</span><span>{{ formataVetor(local) }}</span>
    </div>
</template>

<style scoped>
.coordenadas {
    display: grid;
    grid-template-columns: auto auto;
    justify-content: center;
    gap: 0.15rem 0.9rem;
    margin: -1rem 0 1.5rem;
    font-family: var(--vp-font-family-mono);
    font-size: 0.85rem;
    color: var(--vp-c-text-2);
}

.coordenadas span:nth-child(even) {
    color: var(--vp-c-text-1);
}
</style>
