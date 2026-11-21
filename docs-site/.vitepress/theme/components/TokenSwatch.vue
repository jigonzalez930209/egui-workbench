<script setup lang="ts">
defineProps<{
  name: string
  value: string
}>()
</script>

<template>
  <span class="wb-swatch">
    <i :style="{ background: value }" />
    <code>{{ name }}</code>
    <code>{{ value }}</code>
  </span>
</template>
