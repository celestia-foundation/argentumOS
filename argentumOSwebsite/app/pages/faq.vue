<script setup lang="ts">
const { data: doc } = await useAsyncData('content-faq', () =>
  queryCollection('content').path('/faq').first()
)

useHead({ title: 'FAQ · argentumOS' })
</script>

<template>
  <article class="page page--markdown">
    <ContentRenderer v-if="doc" :value="doc" />
    <p v-else>The questions ate themselves. Try again later.</p>
  </article>
</template>
