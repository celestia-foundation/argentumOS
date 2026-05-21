<script setup lang="ts">
const { data: doc } = await useAsyncData('content-install', () =>
  queryCollection('content').path('/install').first()
)

useHead({ title: 'Install argentumOS' })
</script>

<template>
  <article class="page page--markdown">
    <ContentRenderer v-if="doc" :value="doc" />
    <p v-else>Couldn't find the install page. That's on us.</p>
  </article>
</template>
