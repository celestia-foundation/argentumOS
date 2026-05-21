<script setup lang="ts">
const { data: doc } = await useAsyncData('content-index', () =>
  queryCollection('content').path('/index').first()
)

useHead({
  title: 'argentumOS — a Linux desktop that behaves'
})
</script>

<template>
  <div>
    <HeroSection />
    <FeatureGrid />

    <section class="closer container">
      <ScreenshotPlaceholder
        label="System Settings — argentum-settings"
        caption="The native System Settings panel. Rust + GPUI. Real screenshots coming once we stop arguing about which row to focus."
      />
      <article v-if="doc" class="closer__prose">
        <ContentRenderer :value="doc" />
      </article>
      <div class="closer__cta">
        <NuxtLink to="/install" class="btn btn--primary">
          Get the ISO
        </NuxtLink>
        <NuxtLink to="/apps" class="btn btn--ghost">
          See the apps
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<style scoped>
.closer {
  padding: 4rem 1.5rem 2rem;
  max-width: 920px;
}

.closer__prose {
  max-width: 680px;
  margin: 2rem auto 2.5rem;
}

.closer__cta {
  display: flex;
  justify-content: center;
  gap: 0.8rem;
  flex-wrap: wrap;
}
</style>
