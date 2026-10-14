<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena3d, seta3d, linha3d, pontos3d, rotulos3d, vec3 } from './geometry.js'

const { isDark } = useData()

const mundo = {
  origem: vec3(0, 0, 0),
  eixos: [
    { nome: 'x', direcao: vec3(1, 0, 0) },
    { nome: 'y', direcao: vec3(0, 1, 0) },
    { nome: 'z', direcao: vec3(0, 0, 1) },
  ],
  tamanhoEixos: 1.35,
}

const PONTO = vec3(0.9, 0.15, -0.4)

const ORIGEM = vec3(-1.3, 1.0, 1.9)
const FRENTE = PONTO.sub(ORIGEM).normalized()
const DIREITA = FRENTE.cross(mundo.eixos[1].direcao).normalized()

const camera = {
  origem: ORIGEM,
  eixos: [
    { nome: 'direita', direcao: DIREITA },
    { nome: 'cima', direcao: DIREITA.cross(FRENTE) },
    { nome: 'frente', direcao: FRENTE },
  ],
  tamanhoEixos: 0.9,
}

const ALCANCE = { x: [-1.50, 1.70], y: [-0.45, 2.36], z: [-0.60, 2.93] }

const fig = computed(() => {
  const cores = palette(isDark.value)

  const rotulos = [
    ...mundo.eixos.map(({ nome, direcao }) => ({
      text: nome, position: direcao.mul(mundo.tamanhoEixos + 0.14),
    })),
    ...camera.eixos.map(({ nome, direcao }) => ({
      text: nome, position: camera.origem.add(direcao.mul(camera.tamanhoEixos + 0.3)),
    })),
    { text: 'mundo', position: mundo.origem.add(vec3(0, -0.25, 0)) },
    { text: 'câmera', position: ORIGEM.add(camera.eixos[2].direcao.mul(-0.25)) },
    { text: 'alvo', position: PONTO.add(vec3(0, 0.25, 0)) },
  ]

  return {
    data: [
      ...mundo.eixos.flatMap(({ direcao }, n) => seta3d(mundo.origem, direcao.mul(mundo.tamanhoEixos), cores.serie[n], 0.14)),
      ...camera.eixos.flatMap(({ direcao }, n) => seta3d(camera.origem, camera.origem.add(direcao.mul(camera.tamanhoEixos)), cores.serie[n], 0.14)),
      linha3d([mundo.origem, PONTO], { line: { color: cores.fraca, width: 1.4, dash: 'dot' } }),
      linha3d([ORIGEM.add(camera.eixos[2].direcao.mul(camera.tamanhoEixos)), PONTO], {
        line: { color: cores.fraca, width: 1.4, dash: 'dot' },
      }),
      pontos3d([mundo.origem, camera.origem, PONTO], cores.tinta, 6),
      rotulos3d(rotulos, cores.tinta),
    ],
    layout: {
      ...cena3d(vec3(1.6, 1.0, 2.2), ALCANCE),
      margin: { t: 0, r: 0, b: 0, l: 0 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
</template>
