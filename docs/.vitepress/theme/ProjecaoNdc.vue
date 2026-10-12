<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, seta, vec2, xy } from './geometry.js'

const props = defineProps({
  modo: { type: String, required: true },
})

const { isDark } = useData()

const Z_MAX = 3.0
const Y_MAX = 1.25

const PLANO_Z_1 = 1.0
const NDC_X = 5.0
const NDC_ESCALA = 1.35

const MEIA_ALTURA = { perspectiva: 0.5, ortografica: 1.0 }

const PONTOS = [
  { nome: 'a', p: vec2(1.2, 0.5) },
  { nome: 'b', p: vec2(2.8, 0.5) },
  { nome: 'c', p: vec2(2.0, -0.7) },
]

const fig = computed(() => {
  const cores = palette(isDark.value)
  const perspectiva = props.modo === 'perspectiva'
  const meiaAltura = MEIA_ALTURA[props.modo]

  const noPlano = (p) => (perspectiva ? p.y / p.x : p.y)
  const noNdc = (p) => vec2(NDC_X, (noPlano(p) / meiaAltura) * NDC_ESCALA)
  
  const origemDoRaio = (p) => (perspectiva ? vec2(0, 0) : vec2(0, p.y))

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const agrupadoPorNdc = PONTOS.reduce((mapa, { nome, p }) => {
    const valor = (noPlano(p) / meiaAltura).toFixed(2)

    return mapa.set(valor, [...(mapa.get(valor) ?? []), nome])
  }, new Map())

  const rotulos = [
    { text: 'z', position: vec2(Z_MAX + 0.4, 0), textposition: 'middle right' },
    { text: 'y', position: vec2(0, Y_MAX + 0.4), textposition: 'top center' },
    { text: 'câmera', position: vec2(-0.12, 0.18), textposition: 'middle left' },
    { text: `plano z = ${PLANO_Z_1}`, position: vec2(PLANO_Z_1, meiaAltura).nudged_up(0.08), textposition: 'top center' },
    { text: 'mesma altura y', position: vec2(Z_MAX, 0.5).nudged_up(0.08), textposition: 'top right' },
    { text: 'NDC', position: vec2(NDC_X, NDC_ESCALA + 0.3), textposition: 'top center' },
    ...[1, 0, -1].map((t) => ({
      text: `${t}`,
      position: vec2(NDC_X, t * NDC_ESCALA).nudged_right(-0.12),
      textposition: 'middle left',
    })),
    ...PONTOS.map(({ nome, p }) => ({
      text: nome, position: p.nudged_right(0.12), textposition: 'middle right',
    })),
    ...[...agrupadoPorNdc].map(([valor, nomes]) => ({
      text: `${nomes.join(', ')} = ${valor}`,
      position: vec2(NDC_X, Number(valor) * NDC_ESCALA).nudged_right(0.15),
      textposition: 'middle right',
    })),
  ]

  return {
    data: [
      // o volume de visão: um triângulo que abre na perspectiva, uma caixa na ortográfica
      ...(perspectiva
        ? [1, -1].map((s) => linha([vec2(0, 0), vec2(Z_MAX, s * Z_MAX * meiaAltura)], {
            line: { color: cores.fraca, width: 1.2 },
          }))
        : [1, -1].map((s) => linha([vec2(0, s * meiaAltura), vec2(Z_MAX, s * meiaAltura)], {
            line: { color: cores.fraca, width: 1.2 },
          }))),
      // a linha pontilhada marcando a altura de a e b
      linha([vec2(PLANO_Z_1, 0.5), vec2(Z_MAX, 0.5)], {
        line: { color: cores.fraca, width: 1, dash: 'dot' },
      }),
      // o plano z = 1
      linha([vec2(PLANO_Z_1, -meiaAltura), vec2(PLANO_Z_1, meiaAltura)], {
        line: { color: cores.tinta, width: 2 },
      }),
      // a reta NDC
      linha([vec2(NDC_X, -NDC_ESCALA), vec2(NDC_X, NDC_ESCALA)], {
        line: { color: cores.tinta, width: 2 },
      }),
      // as linhas da reta NDC
      ...[1, 0, -1].map((t) => linha(
        [vec2(NDC_X - 0.08, t * NDC_ESCALA), vec2(NDC_X + 0.08, t * NDC_ESCALA)],
        { line: { color: cores.tinta, width: 1.5 } },
      )),
      // o raio de cada ponto
      ...PONTOS.map(({ p }, i) => linha([origemDoRaio(p), p], {
        line: { color: cores.serie[i], width: 1.4 },
      })),
      // os pontos no plano z = 1 e na reta NDC
      ...PONTOS.map(({ p }, i) => ({
        ...xy([p, vec2(PLANO_Z_1, noPlano(p)), noNdc(p)]),
        type: 'scatter', mode: 'markers',
        marker: { size: 9, color: cores.serie[i] },
        hoverinfo: 'skip', showlegend: false,
      })),
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
      ...eixosIguais([-0.95, 6.4], [-1.85, 2.15]),
      annotations: [
        seta(vec2(-0.4, 0), vec2(Z_MAX + 0.3, 0), cores.fraca),
        seta(vec2(0, -1.6), vec2(0, 1.55), cores.fraca),
      ],
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="450" />
</template>
