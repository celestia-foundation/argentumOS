// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  modules: ['@nuxt/content'],
  css: ['~/assets/css/main.css'],
  app: {
    head: {
      title: 'argentumOS — a Linux desktop that behaves',
      htmlAttrs: { lang: 'en' },
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        {
          name: 'description',
          content:
            'argentumOS is a Linux desktop for people who would like their computer to behave normally for once. No terminal spelunking required.'
        },
        { name: 'theme-color', content: '#1C1C1E' }
      ]
    }
  }
})
