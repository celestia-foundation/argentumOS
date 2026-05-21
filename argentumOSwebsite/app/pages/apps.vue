<script setup lang="ts">
const { data: doc } = await useAsyncData('content-apps', () =>
  queryCollection('content').path('/apps').first()
)

useHead({ title: 'Apps · argentumOS' })
</script>

<template>
  <article class="page page--markdown">
    <ContentRenderer v-if="doc" :value="doc" />
    <p v-else>The apps page wandered off.</p>
  </article>
</template>
