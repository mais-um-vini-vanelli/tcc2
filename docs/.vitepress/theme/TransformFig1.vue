<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, linha2d, pontos2d, rotulos2d, arco2d, alfa, vec2 } from './geometry.js'

const { isDark } = useData()

const LADO = 0.6
const QUADRADO = [
  vec2(-LADO, -LADO),
  vec2(LADO, -LADO),
  vec2(LADO, LADO),
  vec2(-LADO, LADO),
]
const CANTO = vec2(LADO, LADO)

const ROTACAO = Math.PI / 4
const TRANSLACAO = vec2(3, 0)

const gira = (p) => p.rotated(ROTACAO)
const translada = (p) => p.add(TRANSLACAO)

const arcoDaRotacao = (p) => {
  const inicio = Math.atan2(p.y, p.x)

  return arco2d(vec2(0, 0), p.length(), inicio, inicio + ROTACAO)
}

const casos = [
  {
    titulo: 'rotaciona, depois translada',
    meio: QUADRADO.map(gira),
    final: QUADRADO.map((p) => translada(gira(p))),
    arco: arcoDaRotacao(CANTO),
    translacao: [gira(CANTO), translada(gira(CANTO))],
    cantos: [CANTO, gira(CANTO), translada(gira(CANTO))],
  },
  {
    titulo: 'translada, depois rotaciona',
    meio: QUADRADO.map(translada),
    final: QUADRADO.map((p) => gira(translada(p))),
    arco: arcoDaRotacao(translada(CANTO)),
    translacao: [CANTO, translada(CANTO)],
    cantos: [CANTO, translada(CANTO), gira(translada(CANTO))],
  },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const painel = (caso, index) => {
    const offset = vec2(index * 5.8, 0)
    const desloca = (pontos) => pontos.map((p) => p.add(offset))
    const legenda = index === 0

    return [
      // os eixos do plano
      linha2d(desloca([vec2(-1, 0), vec2(4, 0)]), { line: { color: cores.fraca, width: 1 } }),
      linha2d(desloca([vec2(0, -1), vec2(0, 3.2)]), { line: { color: cores.fraca, width: 1 } }),
      // o quadrado original
      linha2d(desloca([...QUADRADO, QUADRADO[0]]), {
        line: { color: cores.fraca, width: 1, dash: 'dot' },
      }),
      // o quadrado depois da primeira transformação
      linha2d(desloca([...caso.meio, caso.meio[0]]), {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.25),
        line: { color: cores.fraca, width: 1 },
      }),
      // o quadrado depois da segunda translação
      linha2d(desloca(((pontos) => [...pontos, pontos[0]])(caso.final)), {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.25),
        line: { color: cores.serie[0], width: 1 },
      }),
      // o arco da rotação
      linha2d(desloca(caso.arco), {
        line: { color: cores.serie[1], width: 2, dash: 'dot' },
        name: 'rotação', legendgroup: 'rotação', showlegend: legenda,
      }),
      // o marcador da translação
      linha2d(desloca(caso.translacao), {
        line: { color: cores.serie[2], width: 2, dash: 'dash' },
        name: 'translação', legendgroup: 'translação', showlegend: legenda,
      }),
      pontos2d(desloca(caso.cantos), cores.tinta, 7),
      rotulos2d(
        [{ text: caso.titulo, position: vec2(1.5, 3.6).add(offset), textposition: 'top center' }],
        cores.tinta,
      ),
    ]
  }

  return {
    data: casos.flatMap(painel),
    layout: cena2d([-1.3, 10.0], [-1.5, 4.4], {
      legend: {
        orientation: 'h', x: 0.5, xanchor: 'center', y: -0.02, yanchor: 'top',
        font: { color: cores.tinta, size: 12 },
      },
      margin: { t: 8, r: 8, b: 34, l: 8 },
    }),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="370" />
</template>
