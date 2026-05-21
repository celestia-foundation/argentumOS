<template>
  <footer class="footer">
    <div class="footer__inner container">
      <div class="footer__left">
        <div class="footer__brand">argentumOS</div>
        <div class="footer__tag">
          A Linux desktop for people who would like their computer to behave normally for once.
        </div>
      </div>
      <div class="footer__right">
        <a href="https://github.com" target="_blank" rel="noopener">GitHub</a>
        <span class="footer__dot">·</span>
        <span class="footer__credit">
          hosted in Celestia Foundation
        </span>
      </div>
    </div>
    <div class="footer__bottom container">
      <span>made by sinisterMage. xoxo Celestia.</span>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  margin-top: 6rem;
  padding: 3rem 0 2rem;
  background: var(--sidebar);
  border-top: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 0.9rem;
}

.footer__inner {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 2rem;
  flex-wrap: wrap;
}

.footer__brand {
  color: var(--text);
  font-weight: 600;
  margin-bottom: 0.4rem;
}

.footer__tag {
  max-width: 360px;
  line-height: 1.5;
}

.footer__right {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.footer__dot { color: var(--border); }

.footer__bottom {
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border);
  font-size: 0.82rem;
  color: var(--text-muted);
}
</style>
