<script setup lang="ts">
import {
  computed,
  onMounted,
  onBeforeUnmount,
  provide,
  reactive,
  ref,
  watch
} from 'vue';

type TabKey = 'spec' | 'solidity' | 'csharp';

interface Entry {
  uid: symbol;
  id: string;
  title: string;
  eip?: string;
  status?: string;
  neoMapping?: string;
  category?: string;
  parityLabel?: string;
  parityClass?: string;
}

const entries = reactive<Entry[]>([]);
const activeId = ref<string | null>(null);
const activeTab = ref<TabKey>('spec');

function register(entry: Entry) {
  if (!entries.find((e) => e.uid === entry.uid)) {
    entries.push(entry);
    if (activeId.value === null) {
      activeId.value = entry.id;
    }
  }
}

function unregister(uid: symbol) {
  const idx = entries.findIndex((e) => e.uid === uid);
  if (idx >= 0) entries.splice(idx, 1);
}

provide('standards-mirror:register', register);
provide('standards-mirror:unregister', unregister);
provide('standards-mirror:active-id', activeId);
provide('standards-mirror:active-tab', activeTab);

const TAB_LABELS: Record<TabKey, string> = {
  spec: 'ERC / EIP Detail',
  solidity: 'Solidity Implementation',
  csharp: 'Neo C# Implementation'
};

const TAB_HINT: Record<TabKey, string> = {
  spec: 'Why the standard exists and how it works on Ethereum.',
  solidity: 'Reference Solidity (Ethereum) implementation.',
  csharp: 'Idiomatic Neo N3 implementation in C# using the Neo SmartContract Framework.'
};

const groupedEntries = computed(() => {
  const groups: Record<string, Entry[]> = {};
  for (const e of entries) {
    const key = e.category ?? 'Standards';
    (groups[key] ||= []).push(e);
  }
  return Object.entries(groups);
});

const activeEntry = computed(() => entries.find((e) => e.id === activeId.value) ?? null);

function selectEntry(id: string) {
  activeId.value = id;
  syncHash();
}

function selectTab(tab: TabKey) {
  activeTab.value = tab;
  syncHash();
}

function syncHash() {
  if (typeof window === 'undefined' || !activeId.value) return;
  const hash = `#${activeId.value}:${activeTab.value}`;
  if (window.location.hash !== hash) {
    history.replaceState(null, '', hash);
  }
}

function applyHash() {
  if (typeof window === 'undefined') return;
  const raw = window.location.hash.replace(/^#/, '');
  if (!raw) return;
  const [id, tab] = raw.split(':');
  if (id && entries.find((e) => e.id === id)) activeId.value = id;
  if (tab === 'spec' || tab === 'solidity' || tab === 'csharp') activeTab.value = tab;
}

const onHashChange = () => applyHash();

onMounted(() => {
  applyHash();
  window.addEventListener('hashchange', onHashChange);
});

onBeforeUnmount(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('hashchange', onHashChange);
  }
});

watch(
  () => entries.length,
  () => applyHash(),
  { flush: 'post' }
);
</script>

<template>
  <div class="standards-mirror" :class="{ 'sm-single': entries.length <= 1 }">
    <div class="sm-shell">
      <aside class="sm-list" v-if="entries.length > 1">
        <header class="sm-list-header">
          <span class="sm-list-title">Ethereum Standards</span>
          <span class="sm-list-count">{{ entries.length }}</span>
        </header>

        <div v-for="[group, list] in groupedEntries" :key="group" class="sm-group">
          <div class="sm-group-label">{{ group }}</div>
          <button
            v-for="entry in list"
            :key="entry.id"
            type="button"
            class="sm-list-item"
            :class="{ active: entry.id === activeId }"
            @click="selectEntry(entry.id)"
          >
            <span class="sm-list-eip" v-if="entry.eip">EIP {{ entry.eip }}</span>
            <span class="sm-list-name">{{ entry.title }}</span>
            <span
              v-if="entry.parityLabel"
              class="sm-pill"
              :class="entry.parityClass || 'sm-pill-default'"
            >
              {{ entry.parityLabel }}
            </span>
          </button>
        </div>

        <footer class="sm-list-footer">
          Every Ethereum proposal — mirrored on Neo N3.
        </footer>
      </aside>

      <section class="sm-detail">
        <header class="sm-detail-header" v-if="activeEntry">
          <div class="sm-detail-titles">
            <div class="sm-detail-eyebrow">
              <span v-if="activeEntry.eip">EIP {{ activeEntry.eip }}</span>
              <span v-if="activeEntry.status">· {{ activeEntry.status }}</span>
              <span v-if="activeEntry.neoMapping">· Neo: {{ activeEntry.neoMapping }}</span>
            </div>
            <h2 class="sm-detail-title">{{ activeEntry.title }}</h2>
          </div>
          <nav class="sm-tabs" role="tablist" aria-label="Implementation views">
            <button
              v-for="key in (['spec', 'solidity', 'csharp'] as TabKey[])"
              :key="key"
              type="button"
              role="tab"
              :aria-selected="activeTab === key"
              class="sm-tab"
              :class="{ active: activeTab === key, [`tab-${key}`]: true }"
              @click="selectTab(key)"
            >
              <span class="sm-tab-dot" :class="`dot-${key}`" aria-hidden="true" />
              {{ TAB_LABELS[key] }}
            </button>
          </nav>
        </header>

        <p class="sm-tab-hint" v-if="activeEntry">{{ TAB_HINT[activeTab] }}</p>

        <div class="sm-panel">
          <slot />
        </div>

        <footer class="sm-detail-footer" v-if="activeEntry">
          <span class="sm-footer-key">Why this matters</span>
          <span class="sm-footer-text">
            Neo N3 supports every meaningful Ethereum standard — sometimes via a direct NEP, sometimes
            via a stronger native primitive. This module lets you compare them side-by-side.
          </span>
        </footer>
      </section>
    </div>
  </div>
</template>

<style scoped>
.standards-mirror {
  margin: 1.5rem 0 3rem;
}

.sm-shell {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  align-items: stretch;
  border: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 16%, transparent);
  border-radius: 14px;
  padding: 1rem;
  background: color-mix(in srgb, var(--vp-c-brand-1) 3%, var(--vp-c-bg-soft));
}

/* Single-entry pages (every individual ERC page): the side list is hidden,
   so the detail panel takes the full content width. No 2-column squeeze. */
.standards-mirror.sm-single .sm-detail {
  width: 100%;
}

.sm-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  background: var(--vp-c-bg);
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 18%, transparent);
  padding: 0.75rem;
  position: sticky;
  top: 80px;
  max-height: calc(100vh - 100px);
  overflow-y: auto;
}

.sm-list-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding: 0.25rem 0.5rem 0.5rem;
  border-bottom: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
}

.sm-list-title {
  font-family: 'Space Grotesk', var(--vp-font-family-base);
  font-weight: 700;
  font-size: 0.95rem;
  letter-spacing: -0.01em;
}

.sm-list-count {
  font-family: var(--vp-font-family-mono);
  font-size: 0.75rem;
  background: color-mix(in srgb, var(--vp-c-brand-1) 14%, transparent);
  color: var(--vp-c-brand-1);
  border-radius: 999px;
  padding: 0.1rem 0.55rem;
}

.sm-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.sm-group-label {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--vp-c-text-2);
  padding: 0.5rem 0.5rem 0.25rem;
}

.sm-list-item {
  text-align: left;
  width: 100%;
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 0.5rem;
  padding: 0.55rem 0.65rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  font: inherit;
  color: var(--vp-c-text-1);
  cursor: pointer;
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    transform 0.15s ease;
}

.sm-list-item:hover {
  background: color-mix(in srgb, var(--vp-c-brand-1) 6%, transparent);
  border-color: color-mix(in srgb, var(--vp-c-brand-1) 18%, transparent);
}

.sm-list-item.active {
  background: color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
  border-color: var(--vp-c-brand-1);
  transform: translateX(2px);
}

.sm-list-eip {
  font-family: var(--vp-font-family-mono);
  font-size: 0.7rem;
  color: var(--vp-c-text-2);
  background: color-mix(in srgb, var(--vp-c-brand-1) 8%, transparent);
  border-radius: 4px;
  padding: 0.1rem 0.35rem;
}

.sm-list-item.active .sm-list-eip {
  color: var(--vp-c-brand-1);
  background: color-mix(in srgb, var(--vp-c-brand-1) 18%, transparent);
}

.sm-list-name {
  font-size: 0.88rem;
  font-weight: 500;
}

.sm-pill {
  font-family: var(--vp-font-family-mono);
  font-size: 0.65rem;
  padding: 0.1rem 0.4rem;
  border-radius: 999px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  white-space: nowrap;
}

.sm-pill-default,
.sm-pill-direct {
  background: color-mix(in srgb, var(--vp-c-brand-1) 16%, transparent);
  color: var(--vp-c-brand-1);
}

.sm-pill-native {
  background: color-mix(in srgb, #6ea8ff 16%, transparent);
  color: #4d8bff;
}

.dark .sm-pill-native {
  color: #87b3ff;
}

.sm-pill-pattern {
  background: color-mix(in srgb, #ffb14e 16%, transparent);
  color: #c97a14;
}

.dark .sm-pill-pattern {
  color: #ffc874;
}

.sm-pill-port {
  background: color-mix(in srgb, #b389ff 18%, transparent);
  color: #6f47d8;
}

.dark .sm-pill-port {
  color: #c4a8ff;
}

.sm-list-footer {
  font-size: 0.7rem;
  color: var(--vp-c-text-2);
  padding: 0.5rem;
  border-top: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
  text-align: center;
  font-style: italic;
}

.sm-detail {
  background: var(--vp-c-bg);
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 18%, transparent);
  padding: 1.25rem 1.4rem 1.4rem;
  min-height: 480px;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 0;
}

.sm-detail-header {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  border-bottom: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
  padding-bottom: 0.85rem;
}

.sm-detail-titles {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.sm-detail-eyebrow {
  font-family: var(--vp-font-family-mono);
  font-size: 0.78rem;
  color: var(--vp-c-text-2);
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.sm-detail-eyebrow span:first-child {
  color: var(--vp-c-brand-1);
  font-weight: 600;
}

.sm-detail-title {
  font-family: 'Space Grotesk', var(--vp-font-family-base);
  font-size: 1.5rem;
  font-weight: 700;
  letter-spacing: -0.01em;
  margin: 0;
  border: none !important;
  padding: 0 !important;
}

.sm-tabs {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.4rem;
  background: color-mix(in srgb, var(--vp-c-brand-1) 4%, var(--vp-c-bg-soft));
  border: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
  border-radius: 10px;
  padding: 0.3rem;
}

.sm-tab {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  padding: 0.55rem 0.6rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 7px;
  font: inherit;
  font-size: 0.83rem;
  font-weight: 500;
  color: var(--vp-c-text-2);
  cursor: pointer;
  transition:
    background 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease,
    transform 0.18s ease;
  white-space: nowrap;
}

.sm-tab:hover {
  color: var(--vp-c-text-1);
  background: color-mix(in srgb, var(--vp-c-brand-1) 7%, transparent);
}

.sm-tab.active {
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  border-color: color-mix(in srgb, var(--vp-c-brand-1) 30%, transparent);
  box-shadow: 0 2px 6px color-mix(in srgb, var(--vp-c-brand-1) 8%, transparent);
}

.sm-tab.active.tab-spec {
  border-color: color-mix(in srgb, var(--vp-c-brand-1) 50%, transparent);
}

.sm-tab.active.tab-solidity {
  border-color: color-mix(in srgb, #6ea8ff 50%, transparent);
}

.sm-tab.active.tab-csharp {
  border-color: color-mix(in srgb, #00e599 55%, transparent);
}

.sm-tab-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--vp-c-text-2);
  flex: 0 0 auto;
}

.dot-spec {
  background: var(--vp-c-brand-1);
}

.dot-solidity {
  background: #6ea8ff;
}

.dot-csharp {
  background: #00e599;
}

.sm-tab-hint {
  font-size: 0.85rem;
  color: var(--vp-c-text-2);
  margin: 0;
  padding: 0.2rem 0;
}

.sm-panel {
  flex: 1;
  min-width: 0;
}

.sm-panel :deep(h1),
.sm-panel :deep(h2),
.sm-panel :deep(h3),
.sm-panel :deep(h4) {
  font-family: 'Space Grotesk', var(--vp-font-family-base);
}

.sm-panel :deep(h2) {
  border-top: none !important;
  padding-top: 0 !important;
  margin-top: 1.25rem !important;
  font-size: 1.2rem;
}

.sm-panel :deep(h3) {
  font-size: 1.02rem;
  margin-top: 1.1rem;
}

.sm-panel :deep(div[class*='language-']) {
  margin: 0.9rem 0;
}

.sm-panel :deep(table) {
  font-size: 0.88rem;
}

.sm-detail-footer {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0.75rem;
  padding-top: 0.85rem;
  border-top: 1px solid color-mix(in srgb, var(--vp-c-brand-1) 12%, transparent);
  font-size: 0.82rem;
  color: var(--vp-c-text-2);
}

.sm-footer-key {
  font-family: var(--vp-font-family-mono);
  font-weight: 600;
  color: var(--vp-c-brand-1);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-size: 0.72rem;
  align-self: center;
  padding: 0.15rem 0.4rem;
  background: color-mix(in srgb, var(--vp-c-brand-1) 10%, transparent);
  border-radius: 4px;
}

.sm-footer-text {
  line-height: 1.5;
}

@media (max-width: 960px) {
  .sm-shell {
    grid-template-columns: 1fr;
    padding: 0.75rem;
  }

  .sm-list {
    position: static;
    max-height: none;
  }

  .sm-list-item {
    transform: none !important;
  }

  .sm-tabs {
    grid-template-columns: 1fr;
  }

  .sm-tab {
    justify-content: flex-start;
  }
}
</style>
