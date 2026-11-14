<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, malha3d, pontos3d, rotulos3d, paralelepipedo, vec3 } from './geometry.js'

const { isDark } = useData()

const OBJETOS = [
  [vec3(0.6, 0.5, 0.7), vec3(1.9, 0.8, 1.4), vec3(1.0, 2.0, 1.0)],
  [vec3(1.9, 0.6, 2.4), vec3(2.9, 1.3, 3.1), vec3(2.1, 1.9, 2.6)],
  [vec3(4.0, 0.7, 0.6), vec3(5.2, 0.5, 1.3), vec3(4.6, 1.8, 0.9)],
  [vec3(4.4, 2.1, 2.1), vec3(5.5, 2.4, 2.8), vec3(4.8, 3.2, 2.2)],
]

const FACE = { i: [0], j: [1], k: [2] }

const ALVO = 0
const ACERTO = OBJETOS[ALVO].reduce((a, p) => a.add(p)).mul(1 / 3)
const ORIGEM = vec3(-0.5, 1.9, 2.6)

const BVH = [
  { profundidade: 0, itens: [0, 1, 2, 3] },
  { profundidade: 1, itens: [0, 1] },
  { profundidade: 2, itens: [0] },
  { profundidade: 2, itens: [1] },
  { profundidade: 1, itens: [2, 3] },
  { profundidade: 2, itens: [2] },
  { profundidade: 2, itens: [3] },
].map((no) => ({ ...no, folha: no.itens.length === 1 }))

const MAIOR_PROFUNDIDADE = Math.max(...BVH.map((no) => no.profundidade))

function caixa(itens, folga) {
  const pontos = itens.flatMap((i) => OBJETOS[i])

  const limites = (eixo) => [
    Math.min(...pontos.map((p) => p[eixo])) - folga,
    Math.max(...pontos.map((p) => p[eixo])) + folga,
  ]

  const [x0, x1] = limites('x')
  const [y0, y1] = limites('y')
  const [z0, z1] = limites('z')
  const min = vec3(x0, y0, z0)

  return {
    min,
    max: vec3(x1, y1, z1),
    arestas: paralelepipedo(min, vec3(x1 - x0, 0, 0), vec3(0, y1 - y0, 0), vec3(0, 0, z1 - z0)).arestas,
  }
}

const CAIXAS = BVH.map((no) => ({ ...no, ...caixa(no.itens, 0.14 * (MAIOR_PROFUNDIDADE - no.profundidade)) }))

const fig = computed(() => {
  const cores = palette(isDark.value)

  const caixaRaiz = CAIXAS[0]
  const caixaAcerto = CAIXAS.find((c) => c.folha && c.itens.includes(ALVO))

  return {
    data: [
      ...OBJETOS.map((objeto) => malha3d(objeto, FACE, {
        color: cores.tinta, opacity: 0.45, flatshading: true,
      })),
      ...OBJETOS.map((objeto) => linha3d([...objeto, objeto[0]], {
        line: { color: cores.fraca, width: 1.4 },
      })),
      ...CAIXAS.map((c) => linha3d(c.arestas, {
        line: {
          color: cores.serie[c.profundidade % 3],
          width: c.folha ? 3 : 1.8,
          dash: c.folha ? 'solid' : 'dot',
        },
      })),
      ...seta3d(ORIGEM, ACERTO, cores.tinta, 0.16),
      pontos3d([ACERTO], cores.tinta, 5),
      rotulos3d([
        { text: 'raiz', position: vec3(caixaRaiz.min.x, caixaRaiz.max.y + 0.2, caixaRaiz.max.z) },
        { text: 'folha', position: vec3(caixaAcerto.min.x, caixaAcerto.max.y + 0.2, caixaAcerto.max.z) },
        { text: 'raio', position: ORIGEM.add(vec3(0, 0.25, 0)) },
      ], cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.1, 0.75, 1.7), { x: [-0.7, 5.9], y: [0.1, 3.9], z: [0.2, 3.5] }, 1.95),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="440" />
</template>
