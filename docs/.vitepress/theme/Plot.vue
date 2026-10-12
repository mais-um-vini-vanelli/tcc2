<script setup>
import { onMounted, onUnmounted, shallowRef, watch } from 'vue'
import { useData } from 'vitepress'

const props = defineProps({
  data: { type: Array, required: true },
  layout: { type: Object, default: () => ({}) },
  height: { type: Number, default: 360 }
})

const el = shallowRef(null)
const { isDark } = useData()

let Plotly = null
let grafico = null

function cores() {
  const css = getComputedStyle(document.documentElement)
  return {
    text: css.getPropertyValue('--vp-c-text-1').trim(),
    line: css.getPropertyValue('--vp-c-divider').trim()
  }
}

function theme() {
  const { text, line } = cores()

  return {
    paper_bgcolor: 'rgba(0,0,0,0)',
    plot_bgcolor: 'rgba(0,0,0,0)',
    font: { color: text, size: 13 },
    xaxis: { gridcolor: line, zerolinecolor: line },
    yaxis: { gridcolor: line, zerolinecolor: line },
    margin: { t: 24, r: 24, b: 48, l: 56 }
  }
}

function merge(base, extra) {
  const res = { ...base, ...extra }
  for (const axis of ['xaxis', 'yaxis']) {
    if (base[axis] || extra[axis]) 
        res[axis] = { ...base[axis], ...extra[axis] }
  }
  return res
}

onMounted(async () => {
  const alvo = el.value

  Plotly = (await import('plotly.js-dist-min')).default

  if (el.value !== alvo) return

  grafico = alvo

  Plotly.newPlot(grafico, props.data, merge(theme(), props.layout), {
    displayModeBar: false,
    responsive: true
  })
})

onUnmounted(() => {
  if (grafico) Plotly.purge(grafico)

  grafico = null
})

watch(
  () => [props.data, props.layout],
  () => {
    if (grafico) Plotly.react(grafico, props.data, merge(theme(), props.layout))
  },
  { deep: true }
)

watch(isDark, () => {
  if (grafico) Plotly.relayout(grafico, merge(theme(), props.layout))
})
</script>

<template>
  <div ref="el" class="plot" :style="{ height: height + 'px' }" />
</template>

<style scoped>
.plot {
  width: 100%;
  margin: 1.5rem 0;
}
</style>