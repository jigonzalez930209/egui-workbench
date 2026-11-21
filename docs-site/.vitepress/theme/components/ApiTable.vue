<script setup lang="ts">
interface ApiRow {
  name: string
  type?: string
  default?: string
  description: string
}

withDefaults(
  defineProps<{
    rows: ApiRow[]
    name?: string
    type?: string
    default?: string
    description?: string
  }>(),
  {
    name: 'API',
    type: 'Type',
    default: 'Default',
    description: 'Description',
  },
)
</script>

<template>
  <div class="vp-doc">
    <table class="wb-api-table">
      <thead>
        <tr>
          <th>{{ name }}</th>
          <th>{{ type }}</th>
          <th>{{ default }}</th>
          <th>{{ description }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in rows" :key="row.name">
          <td><code>{{ row.name }}</code></td>
          <td><code v-if="row.type">{{ row.type }}</code></td>
          <td><code v-if="row.default">{{ row.default }}</code></td>
          <td>{{ row.description }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
