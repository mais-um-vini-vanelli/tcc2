<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, vec2 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const escalaY = ref(0.5)

const ORIGEM = vec2(0, 0)
const T = vec2(1, 1)
const N = vec2(-1, 1)

const TAMANHO_SUPERFICIE = 1.8
const TAMANHO_SETA = 1.3
const TAMANHO_CANTO = 0.2

const aplicaEscala = (v, s) => vec2(v.x, v.y * s)

const fig = computed(() => {
  const cores = palette(isDark.value)
  const escala = escalaY.value

  const t = aplicaEscala(T, escala).normalized()
  const nErrada = aplicaEscala(N, escala).normalized()
  const nCerta = vec2(N.x, N.y / escala).normalized()

  const antes = [T.normalized().mul(-TAMANHO_SUPERFICIE), T.normalized().mul(TAMANHO_SUPERFICIE)]
  const depois = [t.mul(-TAMANHO_SUPERFICIE), t.mul(TAMANHO_SUPERFICIE)]

  const rotulos = [
    { text: 'antes', position: T.normalized().mul(TAMANHO_SUPERFICIE).nudge(0.1, 0.12), textposition: 'top center' },
    { text: 'depois', position: depois[1].nudge(0.12, -0.1), textposition: 'bottom center' },
    { text: 'S n', position: nErrada.mul(TAMANHO_SETA).nudge(-0.1, 0.14), textposition: 'top center' },
    { text: 'S⁻¹ n', position: nCerta.mul(TAMANHO_SETA).nudge(-0.12, 0.14), textposition: 'top center' },
    { text: 'n', position: N.normalized().mul(TAMANHO_SETA).nudge(0.0, 0.1), textposition: 'top right' },
  ]

  return {
    data: [
      linha2d(antes, { line: { color: cores.fraca, width: 1.4, dash: 'dot' } }),
      linha2d([ORIGEM, N.normalized().mul(TAMANHO_SETA)], { line: { color: cores.fraca, width: 1.4, dash: 'dot' } }),
      linha2d(depois, { line: { color: cores.tinta, width: 2.4 } }),
      linha2d([t.mul(TAMANHO_CANTO), t.mul(TAMANHO_CANTO).add(nCerta.mul(TAMANHO_CANTO)), nCerta.mul(TAMANHO_CANTO)], {
        line: { color: cores.fraca, width: 1.2 },
      }),
      pontos2d([ORIGEM], cores.tinta, 7),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-2.3, 2.3], [-1.6, 2.1], {
      annotations: [
        seta2d(ORIGEM, nErrada.mul(TAMANHO_SETA), cores.serie[0]),
        seta2d(ORIGEM, nCerta.mul(TAMANHO_SETA), cores.serie[1]),
      ],
    }),
  }
})
</script>

<template>
    <Slider v-model="escalaY" label="escala y" :min="0.2" :max="1" :step="0.05" />
    <Plot :data="fig.data" :layout="fig.layout" :height="400" />
</template>
