<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, alfa, vec2 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const px = ref(-0.6)

const LUZ = vec2(2.2, 2.6)
const CAIXA = { min: vec2(0.2, 0.9), max: vec2(1.2, 1.7) }
const CHAO = [vec2(-2.2, 0), vec2(3.0, 0)]

function acertoCubo(p) {
  const d = LUZ.sub(p)
  let entrada = 0
  let saida = 1

  for (const eixo of ['x', 'y']) {
    const t0 = (CAIXA.min[eixo] - p[eixo]) / d[eixo]
    const t1 = (CAIXA.max[eixo] - p[eixo]) / d[eixo]

    entrada = Math.max(entrada, Math.min(t0, t1))
    saida = Math.min(saida, Math.max(t0, t1))
  }

  return entrada <= saida ? p.add(d.mul(entrada)) : null
}

const fig = computed(() => {
  const cores = palette(isDark.value)

  const p = vec2(px.value, 0)
  const acerto = acertoCubo(p)

  const cantos = [
    CAIXA.min,
    vec2(CAIXA.max.x, CAIXA.min.y),
    CAIXA.max,
    vec2(CAIXA.min.x, CAIXA.max.y),
    CAIXA.min,
  ]

  const rotulos = [
    { text: 'luz', position: LUZ.nudge(0, 0.18), textposition: 'top center' },
    { text: 'p', position: p.nudge(0, -0.12), textposition: 'bottom center' },
    { text: 'cubo', position: CAIXA.min.add(CAIXA.max).mul(0.5), textposition: 'middle center' },
  ]

  return {
    data: [
      linha2d(CHAO, { line: { color: cores.tinta, width: 2 } }),
      linha2d(cantos, {
        fill: 'toself', fillcolor: alfa(cores.serie[2], 0.16), line: { color: cores.serie[2], width: 1.6 },
      }),
      linha2d([p, acerto ?? LUZ], { line: { color: acerto ? cores.serie[0] : cores.serie[1], width: 2 } }),
      ...(acerto ? [
        linha2d([acerto, LUZ], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
        pontos2d([acerto], cores.serie[0], 9),
      ] : []),
      pontos2d([LUZ], cores.serie[1], 11),
      pontos2d([p], cores.tinta, 9),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-2.4, 3.2], [-0.6, 3.2], {
      annotations: acerto ? [] : [seta2d(p, LUZ, cores.serie[1])],
    }),
  }
})
</script>

<template>
    <Slider v-model="px" label="p" :min="-1.6" :max="1.6" :step="0.05" />
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
</template>
