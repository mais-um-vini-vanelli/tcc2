<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, vec2, xy } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const altura = ref(-0.6)

const MIN = vec2(1.6, 0.9)
const MAX = vec2(4.0, 2.4)
const DIRECAO = vec2(1, 0).rotated(Math.PI * 15 / 180)
const QUADRO_X = [-0.5, 6.3]
const QUADRO_Y = [-1.0, 3.3]

const fig = computed(() => {
  const cores = palette(isDark.value)
  const origem = vec2(0, altura.value)
  
  const entraX = (MIN.x - origem.x) / DIRECAO.x
  const saiX = (MAX.x - origem.x) / DIRECAO.x
  const entraY = (MIN.y - origem.y) / DIRECAO.y
  const saiY = (MAX.y - origem.y) / DIRECAO.y
  
  const entrada = Math.max(entraX, entraY, 0)
  const saida = Math.min(saiX, saiY)
  const acerta = entrada <= saida
  
  const tQuadro = Math.min(
    (QUADRO_X[1] - origem.x) / DIRECAO.x,
    (QUADRO_Y[1] - origem.y) / DIRECAO.y,
  )

  const em = (t) => origem.add(DIRECAO.mul(t))

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  // um plano só tem marca quando o cruzamento está à frente da origem
  const marcos = [
    [entraX, cores.serie[0]], [saiX, cores.serie[0]],
    [entraY, cores.serie[1]], [saiY, cores.serie[1]],
  ].filter(([t]) => t >= 0)

  const rotulos = [
    { text: 'slab x', position: vec2((MIN.x + MAX.x) / 2, 3.15), textposition: 'middle center' },
    { text: 'slab y', position: vec2(5.7, (MIN.y + MAX.y) / 2), textposition: 'middle center' },
    { text: 'O', position: origem.nudged_right(-0.12), textposition: 'middle left' },
    { text: `t_entrada = ${entrada.toFixed(2)}`, position: em(entrada).nudged_right(-0.12), textposition: 'top left' },
    { text: `t_saída = ${saida.toFixed(2)}`, position: em(saida).nudged_right(0.12), textposition: 'bottom right' },
  ]

  return {
    data: [
      // as 4 retas pontilhadas que marcam a caixa
      linha([vec2(MIN.x, QUADRO_Y[0]), vec2(MIN.x, QUADRO_Y[1])], {
        line: { color: cores.fraca, width: 1.4, dash: 'dash' },
      }),
      linha([vec2(MAX.x, QUADRO_Y[0]), vec2(MAX.x, QUADRO_Y[1])], {
        line: { color: cores.fraca, width: 1.4, dash: 'dash' },
      }),
      linha([vec2(QUADRO_X[0], MIN.y), vec2(QUADRO_X[1], MIN.y)], {
        line: { color: cores.fraca, width: 1.4, dash: 'dash' },
      }),
      linha([vec2(QUADRO_X[0], MAX.y), vec2(QUADRO_X[1], MAX.y)], {
        line: { color: cores.fraca, width: 1.4, dash: 'dash' },
      }),
      // o quadrado sólido da caixa
      linha([
        vec2(MIN.x, MIN.y), 
        vec2(MAX.x, MIN.y), 
        vec2(MAX.x, MAX.y),
        vec2(MIN.x, MAX.y), 
        vec2(MIN.x, MIN.y),
      ], { line: { color: cores.tinta, width: 1.6 } }),
      // a reta do raio
      linha([origem, em(tQuadro)], { line: { color: cores.fraca, width: 1.2 } }),
      // a parte do raio que passa dentro da caixa
      ...(acerta
      ? [linha([em(entrada), em(saida)], { line: { color: cores.serie[2], width: 4 } })]
      : []),
      // os 4 pontos onde o raio cruza os slabs
      ...marcos.map(([t, cor]) => ({
        ...xy([em(t)]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cor },
        hoverinfo: 'skip', showlegend: false,
      })),
      {
        ...xy([origem]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cores.tinta },
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
      ...eixosIguais(QUADRO_X, QUADRO_Y),
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Slider v-model="altura" label="altura" :min="-1.6" :max="1.9" :step="0.1" />
    <Plot :data="fig.data" :layout="fig.layout" :height="480" />
</template>
