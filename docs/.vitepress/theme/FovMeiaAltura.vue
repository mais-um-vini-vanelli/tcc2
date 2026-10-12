<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, arco, vec2, xy } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const fov = ref(35)

const Y_MAX = 2.5
const Z_MAX = 4.7
const PONTOS = [vec2(2, 0.8), vec2(4, 0.8)]

const fig = computed(() => {
  const cores = palette(isDark.value)
  const meio = (fov.value / 2) * Math.PI / 180
  const meiaAltura = Math.tan(meio)

  const dentro = (p) => p.y <= p.x * meiaAltura

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const rotulos = [
    { text: 'z', position: vec2(Z_MAX + 0.25, 0), textposition: 'middle right' },
    { text: 'y', position: vec2(0, Y_MAX + 0.25), textposition: 'top center' },
    { text: 'fov/2', position: vec2(1.3, 0).rotated(meio / 2), textposition: 'middle right' },
    { text: 'z = 2', position: vec2(2, 0).nudged_up(-0.12), textposition: 'bottom center' },
    { text: 'z = 4', position: vec2(4, 0).nudged_up(-0.12), textposition: 'bottom center' },
    ...PONTOS.map((p) => ({
      text: `topo = ${(p.x * meiaAltura).toFixed(2)}`,
      position: vec2(p.x, p.x * meiaAltura).nudged_right(-0.15),
      textposition: 'top left',
    })),
    ...PONTOS.map((p) => ({
      text: `(y = ${p.y}) ${dentro(p) ? 'dentro' : 'fora'}`,
      position: p.nudged_right(0.14),
      textposition: 'middle right',
    })),
  ]

  return {
    data: [
      // a linha tangente de cima
      linha([vec2(0, 0), vec2(Z_MAX, Z_MAX * meiaAltura)], {
        line: { color: cores.serie[0], width: 2 },
      }),
      // a linha tangente de baixo
      linha([vec2(0, 0), vec2(Z_MAX, -Z_MAX * meiaAltura)], {
        line: { color: cores.serie[0], width: 1.2 },
      }),
      // o ângulo
      linha(arco(vec2(0, 0), 1.15, 0, meio), { line: { color: cores.fraca, width: 1.2 } }),
      // as perperndiculares pontilhadas que passam pelos pontos
      ...PONTOS.map((p) => linha([vec2(p.x, 0), vec2(p.x, p.x * meiaAltura)], {
        line: { color: cores.serie[0], width: 1.2, dash: 'dot' },
      })),
      {
        ...xy(PONTOS),
        type: 'scatter', mode: 'markers',
        marker: {
          size: 11, color: cores.tinta,
          symbol: PONTOS.map((p) => (dentro(p) ? 'circle' : 'circle-open')),
          line: { color: cores.tinta, width: 2 },
        },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy(rotulos.map((r) => r.position)),
        text: rotulos.map((r) => r.text),
        textposition: rotulos.map((r) => r.textposition),
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 13 },
        hoverinfo: 'skip', showlegend: false,
      },
    ],
    layout: {
      ...eixosIguais([-0.45, Z_MAX + 1.1], [-1.15, Y_MAX + 0.4]),
      annotations: [
        seta(vec2(-0.35, 0), vec2(Z_MAX + 0.15, 0), cores.fraca),
        seta(vec2(0, -1.0), vec2(0, Y_MAX + 0.15), cores.fraca),
      ],
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: 'closest',
    },
  }
})
</script>

<template>
    <Slider v-model="fov" label="fov" :min="20" :max="65" sufixo="°" />
    <Plot :data="fig.data" :layout="fig.layout" :height="480" />
</template>
