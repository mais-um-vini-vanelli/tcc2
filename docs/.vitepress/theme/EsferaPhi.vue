<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, arco2d, vec2 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const theta = ref(70)
const phi = ref(55)

const OFFSET_DE_LADO = vec2(0, 0)
const OFFSET_DE_CIMA = vec2(3.7, 0)
const EIXO = 1.3

const fig = computed(() => {
  const cores = palette(isDark.value)

  const theta_rad = theta.value * Math.PI / 180
  const phi_rad = phi.value * Math.PI / 180

  const altura = Math.cos(theta_rad)

  const raio = Math.sin(theta_rad)
  const pontoDeLado = OFFSET_DE_LADO.add(vec2(raio, altura))
  const pontoDeCima = OFFSET_DE_CIMA.add(vec2(raio * Math.cos(phi_rad), raio * Math.sin(phi_rad)))

  const rotulos = [
    { text: 'de lado', position: OFFSET_DE_LADO.nudge(0, 1.95), textposition: 'top center' },
    { text: '+y', position: OFFSET_DE_LADO.nudge(0, EIXO + 0.08), textposition: 'top center' },
    { text: 'xz', position: OFFSET_DE_LADO.nudge(EIXO + 0.08, 0), textposition: 'middle right' },
    { text: `y = cos θ = ${altura.toFixed(2)}`, position: OFFSET_DE_LADO.add(vec2(raio, altura + 0.12)), textposition: 'top right' },
    { text: `r = sen θ = ${raio.toFixed(2)}`, position: OFFSET_DE_LADO.add(vec2(raio, -0.12)), textposition: 'bottom right' },

    { text: 'de cima', position: OFFSET_DE_CIMA.nudge(0, 1.95), textposition: 'top center' },
    { text: '+z', position: OFFSET_DE_CIMA.nudge(0, EIXO + 0.08), textposition: 'top center' },
    { text: '+x', position: OFFSET_DE_CIMA.nudge(EIXO + 0.08, 0), textposition: 'middle right' },
    { text: 'r', position: pontoDeCima.sub(OFFSET_DE_CIMA).label(OFFSET_DE_CIMA, 0.16), textposition: 'middle center' },
    { text: `xz = (${(raio * Math.cos(phi_rad)).toFixed(2)}, ${(raio * Math.sin(phi_rad)).toFixed(2)})`, position: pontoDeCima.nudge(0, 0.12), textposition: 'top center' },
  ]

  return {
    data: [
      // a esfera unitária nas duas vistas
      ...[OFFSET_DE_LADO, OFFSET_DE_CIMA].map((centro) => linha2d(arco2d(centro, 1, 0, 2 * Math.PI), {
        line: { color: cores.fraca, width: 1, dash: 'dot' },
      })),
      // a linha verde da vista de lado
      linha2d([OFFSET_DE_LADO.add(vec2(-raio, altura)), OFFSET_DE_LADO.add(vec2(raio, altura))], {
        line: { color: cores.serie[2], width: 1.6 },
      }),
      // o círculo verde da linha de cima
      linha2d(arco2d(OFFSET_DE_CIMA, raio, 0, 2 * Math.PI), { line: { color: cores.serie[2], width: 1.6 } }),
      // ângulo θ
      linha2d(arco2d(OFFSET_DE_LADO, 0.44, Math.PI / 2 - theta_rad, Math.PI / 2), { line: { color: cores.fraca, width: 1.2 } }),
      // ângulo φ
      linha2d(arco2d(OFFSET_DE_CIMA, 0.44, 0, phi_rad), { line: { color: cores.fraca, width: 1.2 } }),
      // as linhas pontilhadas dos catetos
      linha2d([vec2(pontoDeLado.x, OFFSET_DE_LADO.y), pontoDeLado], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      linha2d([vec2(pontoDeCima.x, OFFSET_DE_CIMA.y), pontoDeCima], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      linha2d([vec2(OFFSET_DE_CIMA.x, pontoDeCima.y), pontoDeCima], { line: { color: cores.fraca, width: 1.2, dash: 'dot' } }),
      pontos2d([pontoDeLado, pontoDeCima], cores.tinta, 10),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-1.8, 5.6], [-1.6, 2.25], {
      annotations: [
        ...[OFFSET_DE_LADO, OFFSET_DE_CIMA].flatMap((centro) => [
          seta2d(centro.nudge(-EIXO, 0), centro.nudge(EIXO, 0), cores.fraca),
          seta2d(centro.nudge(0, -EIXO), centro.nudge(0, EIXO), cores.fraca),
        ]),
        seta2d(OFFSET_DE_LADO, pontoDeLado, cores.serie[0]),
        seta2d(OFFSET_DE_CIMA, pontoDeCima, cores.serie[0]),
      ],
    }),
  }
})
</script>

<template>
    <Slider v-model="theta" label="θ" :min="10" :max="170" sufixo="°" />
    <Slider v-model="phi" label="φ" :min="0" :max="359" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="380" />
</template>
