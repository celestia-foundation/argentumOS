<template>
  <section class="features">
    <div class="container">
      <header class="features__head">
        <h2>What you get</h2>
        <p>
          Opinionated defaults. Built-in apps that match. Zero
          <code>nano</code> required.
        </p>
      </header>
      <div class="features__grid">
        <article
          v-for="(f, i) in features"
          :key="f.title"
          class="feature"
          :class="`feature--accent-${i % 3}`"
        >
          <div class="feature__icon" aria-hidden="true">{{ f.icon }}</div>
          <h3>{{ f.title }}</h3>
          <p>{{ f.body }}</p>
        </article>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
const features = [
  {
    icon: '·',
    title: 'Silent boot',
    body: 'No scrolling kernel logs. Splash, then login. Suspiciously calm. Intentional.'
  },
  {
    icon: '⌘',
    title: 'GUI installer',
    body: 'Calamares with a few custom pages. Pick a disk, pick a name, hydrate. Done.'
  },
  {
    icon: '◌',
    title: 'Native Settings',
    body: 'A custom Rust/GPUI System Settings panel. No save buttons — changes apply on click, because it is the year of our lord.'
  },
  {
    icon: '◐',
    title: 'Native App Store',
    body: 'A first-party Flathub front-end. Search, install, permissions per app. Bring your own remotes if you must.'
  },
  {
    icon: '◇',
    title: 'Flatpak by default',
    body: 'NixOS underneath, Flatpak on top. Apps update without rebuilding your system at 11pm.'
  },
  {
    icon: '✶',
    title: 'A coherent theme',
    body: 'Dark grays. Argentum gold. Calibrated for human retinas, not for screenshotting in a Reddit thread.'
  }
]
</script>

<style scoped>
.features {
  padding: 4rem 0;
}

.features__head {
  text-align: center;
  margin-bottom: 3rem;
}

.features__head h2 {
  margin-top: 0;
}

.features__head p {
  color: var(--text-muted);
  max-width: 50ch;
  margin: 0.6rem auto 0;
}

.features__grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.2rem;
}

.feature {
  position: relative;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.4rem 1.4rem 1.6rem;
  transition: border-color 0.18s ease, transform 0.18s ease,
    box-shadow 0.18s ease;
  --feature-accent: var(--accent);
  --feature-accent-dim: var(--accent-dim);
}

.feature--accent-0 {
  --feature-accent: var(--accent);
  --feature-accent-dim: var(--accent-dim);
}

.feature--accent-1 {
  --feature-accent: var(--accent-2);
  --feature-accent-dim: var(--accent-2-dim);
}

.feature--accent-2 {
  --feature-accent: var(--accent-3);
  --feature-accent-dim: var(--accent-3-dim);
}

.feature:hover {
  border-color: var(--feature-accent);
  transform: translateY(-3px);
  box-shadow: 0 18px 40px -22px var(--feature-accent-dim);
}

.feature__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--feature-accent-dim);
  color: var(--feature-accent);
  font-size: 1.2rem;
  margin-bottom: 0.9rem;
}

.feature h3 {
  margin: 0 0 0.4rem;
  font-size: 1.05rem;
}

.feature p {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.95rem;
  line-height: 1.55;
}
</style>
