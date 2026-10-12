<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import { palette, eixosIguais, alfa, arco, vec2, xy } from './geometry.js'

const { isDark } = useData()

const LADO = 0.6
const QUADRADO = [
  vec2(-LADO, -LADO),
  vec2(LADO, -LADO),
  vec2(LADO, LADO),
  vec2(-LADO, LADO),
]

const ROTACAO = 30 * Math.PI / 180
const ESCALA_X = 2

const gira = (p) => p.rotated(ROTACAO)
const estica = (p) => vec2(p.x * ESCALA_X, p.y)

const montaCanto = (vertices) => {
  const CANTO = 1

  const c = vertices[CANTO]
  const u = vertices[(CANTO + 3) % 4].sub(c).normalized()
  const v = vertices[(CANTO + 1) % 4].sub(c).normalized()
  
  return {
    c,
    u,
    v,
    angulo: Math.acos(u.dot(v)) * 180 / Math.PI
  }
}

const marcaDoCanto = ({ c, u, v, angulo }) => {
  const MARCA = 0.22
  const reto = Math.abs(angulo - 90) < 0.5

  if (reto)
  {
    const paraA = u.mul(MARCA)
    const paraB = v.mul(MARCA)

    return [c.add(paraA), c.add(paraA).add(paraB), c.add(paraB)]
  }
  else
  {
    const de = Math.atan2(u.y, u.x)

    return arco(c, MARCA, de, de + Math.atan2(u.cross(v), u.dot(v)))
  }
}

const casos = [
  { titulo: 'escala, depois rotaciona', final: QUADRADO.map((p) => gira(estica(p))) },
  { titulo: 'rotaciona, depois escala', final: QUADRADO.map((p) => estica(gira(p))) },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const linha = (pontos, extra) => ({
    ...xy(pontos), type: 'scatter', mode: 'lines',
    hoverinfo: 'skip', showlegend: false, ...extra,
  })

  const painel = (caso, index) => {
    const offset = vec2(index * 4.6, 0)
    const desloca = (pontos) => pontos.map((p) => p.add(offset))

    const canto = montaCanto(caso.final)

    return [
      // as retas dos eixos
      linha(desloca([vec2(-1.9, 0), vec2(1.9, 0)]), { line: { color: cores.fraca, width: 1 } }),
      linha(desloca([vec2(0, -1.5), vec2(0, 1.5)]), { line: { color: cores.fraca, width: 1 } }),
      // os quadrados inicial e final
      linha(desloca([...QUADRADO, QUADRADO[0]]), {
        line: { color: cores.fraca, width: 1, dash: 'dot' },
      }),
      linha(desloca([...caso.final, caso.final[0]]), {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.25),
        line: { color: cores.serie[0], width: 1.6 },
      }),

      linha(desloca(marcaDoCanto(canto)), {
        line: { color: cores.serie[1], width: 2 },
      }),
      {
        ...xy([canto.c.add(offset)]),
        type: 'scatter', mode: 'markers',
        marker: { size: 8, color: cores.tinta },
        hoverinfo: 'skip', showlegend: false,
      },
      {
        ...xy([canto.c.add(offset).nudged_right(0.3), vec2(0, 2.0).add(offset)]),
        text: [`${canto.angulo.toFixed(0)}°`, caso.titulo],
        textposition: ['middle right', 'top center'],
        type: 'scatter', mode: 'text',
        textfont: { color: cores.tinta, size: 13 },
        hoverinfo: 'skip', showlegend: false,
      },
    ]
  }

  return {
    data: casos.flatMap(painel),
    layout: {
      ...eixosIguais([-2.1, 6.9], [-1.8, 2.5]),
      margin: { t: 8, r: 8, b: 8, l: 8 },
      hovermode: false,
    },
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="330" />
</template>
