<template>
  <figure class="shot">
    <div class="shot__frame">
      <div class="shot__chrome">
        <span /><span /><span />
      </div>
      <div class="shot__body">
        <div class="shot__label">{{ label }}</div>
      </div>
    </div>
    <figcaption v-if="caption" class="shot__cap">{{ caption }}</figcaption>
  </figure>
</template>

<script setup lang="ts">
defineProps<{
  label: string
  caption?: string
}>()
</script>

<style scoped>
.shot {
  margin: 2rem 0;
}

.shot__frame {
  background: var(--sidebar);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: 0 30px 60px -30px rgba(0, 0, 0, 0.6);
}

.shot__chrome {
  display: flex;
  gap: 6px;
  padding: 10px 14px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
}

.shot__chrome span {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--border);
}

.shot__chrome span:first-child { background: #5a4a3a; }

.shot__body {
  min-height: 260px;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    repeating-linear-gradient(
      45deg,
      transparent 0 14px,
      rgba(200, 169, 126, 0.04) 14px 15px
    ),
    var(--sidebar);
}

.shot__label {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  color: var(--text-muted);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 0.6rem 1rem;
  border: 1px dashed var(--border);
  border-radius: var(--radius);
}

.shot__cap {
  margin-top: 0.8rem;
  font-size: 0.85rem;
  color: var(--text-muted);
  text-align: center;
}
</style>
