<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, pontos3d, rotulos3d, arco3d, vec3 } from './geometry.js'

const { isDark } = useData()

const GRAU = Math.PI / 180

const LUZ = vec3(-1.6, 0, 0)
const EIXO = vec3(1, 0, 0)
const CIMA = vec3(0, 1, 0)
const LADO = vec3(0, 0, 1)

const TAMANHO_EIXO = 1.8

const INTERNO = 20
const EXTERNO = 30

const naBoca = (raio, angulo) => LUZ
  .add(EIXO.mul(TAMANHO_EIXO))
  .add(CIMA.mul(raio * Math.cos(angulo * GRAU)))
  .add(LADO.mul(raio * Math.sin(angulo * GRAU)))

const PONTO = naBoca(0.85, 0)
const paraPonto = PONTO.sub(LUZ).normalized()
const theta = Math.acos(paraPonto.dot(EIXO))
const t = (Math.cos(theta) - Math.cos(EXTERNO * GRAU)) / (Math.cos(INTERNO * GRAU) - Math.cos(EXTERNO * GRAU))

const CONES = [
  { angulo: INTERNO, nome: 'cone interno', slot: 1, onde: 180 },
  { angulo: EXTERNO, nome: 'cone externo', slot: 2, onde: 180 },
]

const ALCANCE = { x: [-1.90, 0.50], y: [-1.34, 1.34], z: [-1.34, 1.34] }

const fig = computed(() => {
  const cores = palette(isDark.value)

  const perpendicular = paraPonto.sub(EIXO.mul(paraPonto.dot(EIXO))).normalized()

  const rotulos = [
    { text: 'luz', position: LUZ.add(vec3(0, 0.2, 0)) },
    { text: `θ = ${(theta / GRAU).toFixed(1)}°`, position: LUZ.add(EIXO.mul(0.8)).add(perpendicular.mul(0.3)) },
    { text: `t = ${t.toFixed(2)}`, position: PONTO.add(vec3(0, 0.14, 0)) },
    ...CONES.map(({ angulo, nome, onde }) => ({
      text: `${nome} (${angulo}°)`,
      position: naBoca(TAMANHO_EIXO * Math.tan(angulo * GRAU), onde),
      textposition: 'bottom center',
    })),
  ]

  return {
    data: [
      ...CONES.flatMap(({ angulo, slot }) => {
        const raio = TAMANHO_EIXO * Math.tan(angulo * GRAU)
        const cor = cores.serie[slot]

        return [
          linha3d(arco3d(LUZ.add(EIXO.mul(TAMANHO_EIXO)), raio, CIMA, LADO), { line: { color: cor, width: 1.6 } }),
          linha3d([0, 90, 180, 270].flatMap((a) => [LUZ, naBoca(raio, a), null]), {
            line: { color: cor, width: 1 },
          }),
        ]
      }),
      linha3d(arco3d(LUZ, 0.5, EIXO, perpendicular, 0, theta), { line: { color: cores.fraca, width: 1.4 } }),
      ...seta3d(LUZ, LUZ.add(EIXO.mul(TAMANHO_EIXO)), cores.fraca),
      ...seta3d(LUZ, PONTO, cores.serie[0]),
      pontos3d([LUZ, PONTO], cores.tinta, 6),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(-0.5, 0.0, 0.7), ALCANCE),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
</template>
