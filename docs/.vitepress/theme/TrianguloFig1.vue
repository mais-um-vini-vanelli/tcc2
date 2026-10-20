<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, vec2 } from './geometry.js'

const { isDark } = useData()

const A = vec2(2, 2), B = vec2(8, 4), P = vec2(4, 7)

const u = B.sub(A)
const v = P.sub(A)

const uNormalizado = u.normalized()
const uLinha = vec2(-u.y, u.x)
const uLinhaNormalizado = uLinha.normalized()

const baseDaPerpendicular = A.add(u.mul(v.dot(u) / u.dot(u)))

const edge = computed(() => {
  const gap = 0.42
  const cores = palette(isDark.value)

  const rotulos = [
    { text: 'A', position: A.nudge(-0.42, 0), textposition: 'middle left' },
    { text: 'B', position: B.nudge(0.42, 0), textposition: 'middle right' },
    { text: 'P', position: P.nudge(0, 0.3), textposition: 'top center' },
    { text: 'u', position: u.label(A, -gap) },
    { text: 'v', position: v.label(A, gap) },
    { text: "u'", position: uLinha.label(A, gap) },
    { text: 'h', position: P.sub(baseDaPerpendicular).label(baseDaPerpendicular, -gap) },
  ]

  return {
    data: [
      // a reta infinita que passa por A e B
      linha2d(
        [A.sub(uNormalizado.mul(1.2)), B.add(uNormalizado.mul(1.2))],
        { line: { color: cores.fraca, width: 1, dash: 'dot' } }
      ),
      // o triângulo ABP
      linha2d(
        [A, B, P, A],
        { fill: 'toself', fillcolor: cores.preenche, line: { color: cores.fraca, width: 1 } }
      ),
      // a altura h, medida na direção de u'
      linha2d(
        [baseDaPerpendicular, P],
        { line: { color: cores.tinta, width: 1.5, dash: 'dash' } }
      ),
      // o ângulo reto da perpendicular
      linha2d(
        [uNormalizado, uNormalizado.add(uLinhaNormalizado), uLinhaNormalizado].map((d) => baseDaPerpendicular.add(d.mul(0.38))),
        { line: { color: cores.fraca, width: 1.2 } }
      ),
      pontos2d([A, B, P], cores.tinta),
      rotulos2d(rotulos, cores.tinta, 15),
    ],
    layout: cena2d([-1.4, 9.4], [0.6, 9.2], {
      annotations: [
        seta2d(A, B, cores.serie[0]),
        seta2d(A, P, cores.serie[1]),
        seta2d(A, A.add(uLinha), cores.serie[2]),
      ],
    }),
  }
})
</script>

<template>
    <Plot :data="edge.data" :layout="edge.layout" />
</template>
