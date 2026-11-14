<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, pontos2d, rotulos2d, vec2 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const escalaX = ref(1.5)
const rotacaoZ = ref(30)
const translacaoY = ref(0.5)

const ORIGEM = vec2(-4.6, 0)
const DIRECAO = vec2(1, 0)

const CAIXA = [vec2(-1, -1), vec2(1, -1), vec2(1, 1), vec2(-1, 1), vec2(-1, -1)]

const acertoCaixa = (origem, direcao) => {
  const entradaSlab = (o, d) => Math.min((-1 - o) / d, (1 - o) / d)

  return Math.max(entradaSlab(origem.x, direcao.x), entradaSlab(origem.y, direcao.y))
}

const tracosT = (origem, direcao, t) => {
  const perpendicular = vec2(-direcao.y, direcao.x).normalized().mul(0.18)

  return Array.from({ length: Math.floor(t) }, (_, i) => {
    const p = origem.add(direcao.mul(i + 1))

    return [p.sub(perpendicular), p.add(perpendicular)]
  })
}

const fig = computed(() => {
  const cores = palette(isDark.value)

  const rotacao = rotacaoZ.value * Math.PI / 180
  const translacao = vec2(0, translacaoY.value)

  const aplicaTransform = (p) => vec2(p.x * escalaX.value, p.y).rotated(rotacao).add(translacao)

  const desfazTransform = (p) => {
    const q = p.rotated(-rotacao)

    return vec2(q.x / escalaX.value, q.y)
  }

  const origemLocal = desfazTransform(ORIGEM.sub(translacao))
  const direcaoLocal = desfazTransform(DIRECAO)

  const t = acertoCaixa(origemLocal, direcaoLocal)

  const paineis = [
    {
      titulo: 'mundo', caixa: CAIXA.map(aplicaTransform),
      origem: ORIGEM, direcao: DIRECAO, offset: vec2(0, 0),
    },
    {
      titulo: 'espaço local', caixa: CAIXA,
      origem: origemLocal, direcao: direcaoLocal, offset: vec2(7.2, 0),
    },
  ]

  const painel = ({ titulo, caixa, origem, direcao, offset }) => {
    const o = origem.add(offset)
    const acerto = o.add(direcao.mul(t))
    const direcaoUnitaria = direcao.normalized()
    const perpendicular = vec2(-direcaoUnitaria.y, direcaoUnitaria.x)

    return [
      linha2d(caixa.map((p) => p.add(offset)), {
        fill: 'toself', fillcolor: cores.preenche,
        line: { color: cores.fraca, width: 1 },
      }),
      linha2d([o, acerto], { line: { color: cores.serie[0], width: 1.4 } }),
      ...tracosT(o, direcao, t).map((traco) => linha2d(traco, { line: { color: cores.serie[0], width: 1.4 } })),
      pontos2d([o], cores.tinta, 7),
      pontos2d([acerto], cores.serie[1], 11),
      rotulos2d([
        { text: titulo, position: offset.nudge(-4.6, 4.0), textposition: 'top right' },
        { text: 'o', position: o.sub(direcaoUnitaria.mul(0.34)) },
        { text: 'd', position: direcao.label(o, 0.3) },
        { text: `t = ${t.toFixed(2)}`, position: acerto.add(perpendicular.mul(0.4)) },
      ], cores.tinta),
    ]
  }

  return {
    data: paineis.flatMap(painel),
    layout: cena2d([-5.3, 8.8], [-3.3, 5.15], {
      annotations: paineis.map(({ origem, direcao, offset }) => {
        const o = origem.add(offset)

        return seta2d(o, o.add(direcao), cores.serie[0])
      }),
    }),
  }
})
</script>

<template>
    <Slider v-model="escalaX" label="escala x" :min="1" :max="2" :step="0.1" />
    <Slider v-model="rotacaoZ" label="rotação z" :min="0" :max="90" :step="5" sufixo="°" />
    <Slider v-model="translacaoY" label="translação y" :min="-0.9" :max="0.9" :step="0.1" />
    <Plot :data="fig.data" :layout="fig.layout" :height="370" />
</template>
