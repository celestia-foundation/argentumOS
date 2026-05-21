<template>
  <header class="nav">
    <div class="nav__inner container">
      <NuxtLink to="/" class="nav__brand">
        <span class="nav__mark">ag</span>
        <span class="nav__word">argentumOS</span>
      </NuxtLink>
      <nav class="nav__links" aria-label="Primary">
        <NuxtLink to="/install">Install</NuxtLink>
        <NuxtLink to="/apps">Apps</NuxtLink>
        <NuxtLink to="/philosophy">Philosophy</NuxtLink>
        <NuxtLink to="/faq">FAQ</NuxtLink>
      </nav>
    </div>
  </header>
</template>

<style scoped>
.nav {
  position: sticky;
  top: 0;
  z-index: 10;
  background: rgba(22, 22, 24, 0.85);
  backdrop-filter: saturate(140%) blur(8px);
  border-bottom: 1px solid var(--border);
}

.nav__inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
}

.nav__brand {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  font-weight: 600;
  color: var(--text);
  border-bottom: none !important;
}

.nav__brand:hover { color: var(--accent); }

.nav__mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 7px;
  background: var(--accent);
  color: var(--bg);
  font-weight: 700;
  font-size: 0.85rem;
  letter-spacing: -0.02em;
}

.nav__word { font-size: 1rem; }

.nav__links {
  display: flex;
  align-items: center;
  gap: 1.4rem;
}

.nav__links a {
  font-size: 0.92rem;
  color: var(--text-muted);
  border-bottom: none;
}

.nav__links a:hover,
.nav__links a.router-link-active {
  color: var(--accent);
}

@media (max-width: 540px) {
  .nav__word { display: none; }
  .nav__links { gap: 1rem; }
  .nav__links a { font-size: 0.85rem; }
}
</style>
