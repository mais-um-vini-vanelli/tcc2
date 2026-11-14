<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, malha3d, pontos3d, rotulos3d, arco3d, alfa, vec3 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const theta = ref(55)

const ORIGEM = vec3(0, 0, 0)
const NORMAL = vec3(0, 1, 0)
const EIXO_X = vec3(1, 0, 0)
const EIXO_Z = vec3(0, 0, 1)

const RAIO = 1.5
const TELA = 1.0
const METADE = 0.5

const QUAD_INDICES = { i: [0, 0], j: [1, 2], k: [2, 3] }

const QUAD = [
  vec3(-METADE, 0, -METADE),
  vec3(METADE, 0, -METADE),
  vec3(METADE, 0, METADE),
  vec3(-METADE, 0, METADE),
]

const PLANO = [
  vec3(-1.6, 0, -1.6),
  vec3(1.6, 0, -1.6),
  vec3(1.6, 0, 1.6),
  vec3(-1.6, 0, 1.6),
]

const ALCANCE = { x: [-1.75, 1.75], y: [-0.08, 1.72], z: [-1.75, 1.75] }

const cosTheta = computed(() => Math.cos(theta.value * Math.PI / 180))

const fig = computed(() => {
  const cores = palette(isDark.value)

  const t = theta.value * Math.PI / 180
  const v = vec3(Math.sin(t), Math.cos(t), 0)
  const u = v.cross(EIXO_Z)

  const olho = v.mul(RAIO)

  const projetado = QUAD.map((p) => v.mul(TELA)
    .add(u.mul(p.dot(u)))
    .add(EIXO_Z.mul(p.dot(EIXO_Z))))

  const rotulos = [
    { text: 'A = 1', position: vec3(-METADE - 0.08, 0, METADE + 0.08) },
    // acima da borda de cima do projetado, que é o lado -u
    { text: 'A · cos θ', position: v.mul(TELA).add(u.mul(-(METADE * Math.cos(t) + 0.3))) },
    { text: 'n', position: NORMAL.mul(1.26) },
    { text: 'olho', position: olho.add(v.mul(0.22)) },
    { text: 'θ', position: arco3d(ORIGEM, 0.66, NORMAL, EIXO_X, 0, t / 2).at(-1) },
  ]

  return {
    data: [
      malha3d(PLANO, QUAD_INDICES, { color: cores.tinta, opacity: 0.05, flatshading: true }),
      linha3d(arco3d(ORIGEM, RAIO, EIXO_X, EIXO_Z), { line: { color: cores.fraca, width: 1 } }),
      linha3d(arco3d(ORIGEM, RAIO, EIXO_X, NORMAL, 0, Math.PI), { line: { color: cores.fraca, width: 1 } }),
      linha3d(arco3d(ORIGEM, RAIO, EIXO_Z, NORMAL, 0, Math.PI), { line: { color: cores.fraca, width: 1 } }),
      // o quadrado fixo na superfície
      malha3d(QUAD, QUAD_INDICES, { color: cores.serie[0], opacity: 0.3, flatshading: true }),
      linha3d([...QUAD, QUAD[0]], { line: { color: cores.serie[0], width: 2 } }),
      // o quadrado projetado
      malha3d(projetado, QUAD_INDICES, { color: cores.serie[1], opacity: 0.3, flatshading: true }),
      linha3d([...projetado, projetado[0]], { line: { color: cores.serie[1], width: 2 } }),
      linha3d(QUAD.flatMap((p, n) => [p, projetado[n], null]), {
        line: { color: alfa(cores.serie[0], 0.6), width: 1.4, dash: 'dot' },
      }),
      linha3d(arco3d(ORIGEM, 0.66, NORMAL, EIXO_X, 0, t), { line: { color: cores.fraca, width: 1.4 } }),
      ...seta3d(ORIGEM, NORMAL.mul(1.18), cores.fraca),
      ...seta3d(ORIGEM, olho, cores.serie[2]),
      pontos3d([olho], cores.tinta, 6),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(0.9, 0.75, 1.8), ALCANCE, 2.05),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Slider v-model="theta" label="θ" :min="0" :max="75" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
    <p class="legenda">cos θ = {{ cosTheta.toFixed(2) }}</p>
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
