<script setup>
defineProps({
  label: { type: String, required: true },
  min: { type: Number, required: true },
  max: { type: Number, required: true },
  step: { type: Number, default: 1 },
  sufixo: { type: String, default: '' },
})

const valor = defineModel({ type: Number, required: true })
</script>

<template>
  <label class="controle">
    <span class="nome">{{ label }}</span>
    <input type="range" :min="min" :max="max" :step="step" v-model.number="valor">
    <span class="valor">{{ valor }}{{ sufixo }}</span>
  </label>
</template>

<style scoped>
.controle {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin: 1.5rem 0 -1rem;
  font-size: 0.85rem;
  color: var(--vp-c-text-2);
}

.nome {
  min-width: 4.5rem;
}

input {
  flex: 1;
  max-width: 16rem;
  accent-color: var(--vp-c-brand-1);
}

.valor {
  min-width: 4rem;
  font-family: var(--vp-font-family-mono);
  color: var(--vp-c-text-1);
}
</style>
