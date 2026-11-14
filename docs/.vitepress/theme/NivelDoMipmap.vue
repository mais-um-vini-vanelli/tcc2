<script setup>
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import { palette, cena2d, seta2d, linha2d, rotulos2d, alfa, vec2 } from './geometry.js'
import Slider from './Slider.vue'

const { isDark } = useData()

const texelsDx = ref(0.9)
const texelsDy = ref(2)

const TEXTURA = 8
const NIVEIS = Math.log2(TEXTURA)

const DIR_DX = vec2(1, 0.18).normalized()
const DIR_DY = vec2(0.22, 1).normalized()

const numeros = computed(() => {
  const maior = Math.max(texelsDx.value, texelsDy.value)
  const lambda = Math.min(Math.max(Math.log2(maior), 0), NIVEIS)

  return {
    lambda: lambda.toFixed(2),
    nivel: Math.round(lambda),
  }
})

function grid(passo, extra) {
  const linhas = []

  for (let i = 0; i <= TEXTURA; i += passo) {
    linhas.push(vec2(i, 0), vec2(i, TEXTURA), null, vec2(0, i), vec2(TEXTURA, i), null)
  }

  return linha2d(linhas, extra)
}

const fig = computed(() => {
  const cores = palette(isDark.value)

  const { nivel } = numeros.value
  const lado = 2 ** nivel

  const canto = Math.floor(TEXTURA / 2 / lado) * lado
  const texel = [
    vec2(canto, canto),
    vec2(canto + lado, canto),
    vec2(canto + lado, canto + lado),
    vec2(canto, canto + lado),
    vec2(canto, canto),
  ]

  const dx = DIR_DX.mul(texelsDx.value)
  const dy = DIR_DY.mul(texelsDy.value)

  const centroTexel = vec2(canto + lado / 2, canto + lado / 2)
  const base = centroTexel.sub(dx.add(dy).mul(0.5))
  const area = [base, base.add(dx), base.add(dx).add(dy), base.add(dy), base]

  const rotulos = [
    { text: 'dx', position: dx.label(base, -0.3) },
    { text: 'dy', position: dy.label(base, 0.3) },
  ]

  return {
    data: [
      grid(1, { line: { color: alfa(cores.tinta, 0.16), width: 1 } }),
      grid(lado, { line: { color: alfa(cores.serie[1], 0.45), width: 1.2 } }),
      linha2d([vec2(0, 0), vec2(TEXTURA, 0), vec2(TEXTURA, TEXTURA), vec2(0, TEXTURA), vec2(0, 0)], {
        line: { color: cores.tinta, width: 1.6 },
      }),
      linha2d(texel, {
        fill: 'toself', fillcolor: alfa(cores.serie[1], 0.18), line: { color: cores.serie[1], width: 1.8 },
      }),
      linha2d(area, {
        fill: 'toself', fillcolor: alfa(cores.serie[0], 0.16), line: { color: cores.serie[0], width: 1.8 },
      }),
      rotulos2d(rotulos, cores.tinta),
    ],
    layout: cena2d([-1.3, 9.3], [-1.3, 9.3], {
      annotations: [
        seta2d(base, base.add(dx), cores.serie[0]),
        seta2d(base, base.add(dy), cores.serie[0]),
      ],
    }),
  }
})
</script>

<template>
    <Slider v-model="texelsDx" label="dx" :min="0.5" :max="8" :step="0.1" sufixo=" texels" />
    <Slider v-model="texelsDy" label="dy" :min="0.5" :max="8" :step="0.1" sufixo=" texels" />
    <Plot :data="fig.data" :layout="fig.layout" :height="420" />
    <p class="legenda">λ = {{ numeros.lambda }} &nbsp;&nbsp; nível {{ numeros.nivel }}</p>
</template>

<style scoped>
.legenda {
    margin: -1rem 0 1.5rem;
    text-align: center;
    font-family: var(--vp-font-family-mono);
    font-size: 0.85rem;
    color: var(--vp-c-text-2);
}
</style>
