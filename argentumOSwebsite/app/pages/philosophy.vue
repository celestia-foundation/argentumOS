<script setup lang="ts">
const { data: doc } = await useAsyncData('content-philosophy', () =>
  queryCollection('content').path('/philosophy').first()
)

useHead({ title: 'Philosophy · argentumOS' })
</script>

<template>
  <article class="page page--markdown">
    <ContentRenderer v-if="doc" :value="doc" />
    <p v-else>No philosophy today, sorry.</p>
  </article>
</template>
