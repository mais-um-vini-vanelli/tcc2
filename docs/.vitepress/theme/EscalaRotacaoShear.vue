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

    return arco2d(c, MARCA, de, de + Math.atan2(u.cross(v), u.dot(v)))
  }
}

const casos = [
  { titulo: 'escala, depois rotaciona', final: QUADRADO.map((p) => gira(estica(p))) },
  { titulo: 'rotaciona, depois escala', final: QUADRADO.map((p) => estica(gira(p))) },
]

const fig = computed(() => {
  const cores = palette(isDark.value)

  const painel = (caso, index) => {
    const offset = vec2(index * 4.6, 0)
    const desloca = (pontos) => pontos.map((p) => p.add(offset))

    const canto = montaCanto(caso.final)

    const rotulos = [
      {
        text: `${canto.angulo.toFixed(0)}°`,
        position: canto.c.add(offset).nudge(0.3, 0),
        textposition: 'middle right',
      },
      { text: caso.titulo, position: vec2(0, 2.0).add(offset), textposition: 'top center' },
    ]

    return [
      // as retas dos eixos
      linha2d(desloca([vec2(-1.9, 0), vec2(1.9, 0)]), { line: { color: cores.fraca, width: 1 } }),
      linha2d(desloca([vec2(0, -1.5), vec2(0, 1.5)]), { line: { color: cores.fraca, width: 1 } }),
      // os quadrados inicial e final
      linha2d(desloca([...QUADRADO, QUADRADO[0]]), {
        line: { color: cores.fraca, width: 1, dash: 'dot' },
      }),
      linha2d(desloca([...caso.final, caso.final[0]]), {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.25),
        line: { color: cores.serie[0], width: 1.6 },
      }),

      linha2d(desloca(marcaDoCanto(canto)), {
        line: { color: cores.serie[1], width: 2 },
      }),
      pontos2d([canto.c.add(offset)], cores.tinta, 8),
      rotulos2d(rotulos, cores.tinta),
    ]
  }

  return {
    data: casos.flatMap(painel),
    layout: cena2d([-2.1, 6.9], [-1.8, 2.5]),
  }
})
</script>

<template>
    <Plot :data="fig.data" :layout="fig.layout" :height="330" />
</template>
