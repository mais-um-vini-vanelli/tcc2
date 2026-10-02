<script setup>
import { onMounted, onUnmounted, shallowRef, watch } from 'vue'
import { useData } from 'vitepress'

const props = defineProps({
  data: { type: Array, required: true },
  layout: { type: Object, default: () => ({}) }
})

const el = shallowRef(null)
const { isDark } = useData()

let Plotly = null

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
  Plotly = (await import('plotly.js-dist-min')).default
  Plotly.newPlot(el.value, props.data, merge(theme(), props.layout), {
    displayModeBar: false,
    responsive: true
  })
})

onUnmounted(() => Plotly?.purge(el.value))

watch(
  () => [props.data, props.layout],
  () => Plotly?.react(el.value, props.data, merge(theme(), props.layout)),
  { deep: true }
)

watch(isDark, () => {
  if (!Plotly) return

  const { text, line } = cores()

  Plotly.relayout(el.value, merge(theme(), props.layout))
})
</script>

<template>
  <div ref="el" class="plot" />
</template>

<style scoped>
.plot {
  width: 100%;
  height: 360px;
  margin: 1.5rem 0;
}
</style>