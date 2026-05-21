<template>
  <section class="hero">
    <div class="hero__inner container">
      <span class="tag">v0.1 · it exists, or will, probably</span>
      <h1 class="hero__title">
        A Linux desktop that <span class="accent-grad">behaves</span>.
      </h1>
      <p class="hero__lede">
        You install it. You log in. Things mostly function. No ritual incantations,
        no terminal spelunking, no "quick fix" that turns into a three-hour side quest.
      </p>
      <div class="hero__cta">
        <NuxtLink to="/install" class="btn btn--primary">
          Get argentumOS
          <span aria-hidden="true">→</span>
        </NuxtLink>
        <NuxtLink to="/philosophy" class="btn btn--ghost">
          Why this exists
        </NuxtLink>
      </div>
      <p class="hero__sub">
        Built on NixOS so it doesn't fall apart. Looks nothing like NixOS so you
        don't have to either.
      </p>
    </div>
    <div class="hero__halo" aria-hidden="true" />
  </section>
</template>

<style scoped>
.hero {
  position: relative;
  padding: 5rem 0 4rem;
  overflow: hidden;
}

.hero__inner {
  position: relative;
  z-index: 1;
  text-align: center;
}

.hero__title {
  font-size: clamp(2.4rem, 6vw, 4rem);
  font-weight: 700;
  letter-spacing: -0.025em;
  margin: 1.2rem auto 1.2rem;
  max-width: 18ch;
  line-height: 1.05;
}

.accent-grad {
  background: linear-gradient(
    100deg,
    var(--accent) 0%,
    var(--accent-hover) 35%,
    var(--accent-2) 100%
  );
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  font-style: italic;
}

.hero__lede {
  font-size: 1.15rem;
  color: var(--text-muted);
  max-width: 60ch;
  margin: 0 auto 2rem;
  line-height: 1.55;
}

.hero__cta {
  display: flex;
  justify-content: center;
  gap: 0.8rem;
  flex-wrap: wrap;
  margin-bottom: 2.2rem;
}

.hero__sub {
  color: var(--text-muted);
  font-size: 0.92rem;
  max-width: 50ch;
  margin: 0 auto;
}

.hero__halo {
  position: absolute;
  top: -25%;
  left: 50%;
  transform: translateX(-50%);
  width: 110%;
  height: 720px;
  background:
    radial-gradient(
      circle at 30% 35%,
      rgba(242, 181, 68, 0.35) 0%,
      rgba(242, 181, 68, 0.05) 30%,
      transparent 55%
    ),
    radial-gradient(
      circle at 72% 30%,
      rgba(157, 124, 255, 0.32) 0%,
      rgba(157, 124, 255, 0.05) 30%,
      transparent 55%
    ),
    radial-gradient(
      circle at 55% 75%,
      rgba(52, 211, 163, 0.18) 0%,
      transparent 50%
    );
  filter: blur(50px);
  pointer-events: none;
  z-index: 0;
  animation: halo-drift 18s ease-in-out infinite alternate;
}

@keyframes halo-drift {
  from { transform: translateX(-50%) translateY(0) scale(1); }
  to   { transform: translateX(-50%) translateY(-20px) scale(1.04); }
}

@media (prefers-reduced-motion: reduce) {
  .hero__halo { animation: none; }
}
</style>
