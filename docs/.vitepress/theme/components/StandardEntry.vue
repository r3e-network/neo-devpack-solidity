<script setup lang="ts">
import { computed, inject, onBeforeUnmount, onMounted, type Ref } from 'vue';

interface Props {
  id: string;
  title: string;
  eip?: string;
  status?: string;
  neoMapping?: string;
  category?: string;
  parityLabel?: string;
  parityClass?: string;
}

const props = defineProps<Props>();

const register = inject<(entry: any) => void>('standards-mirror:register');
const unregister = inject<(uid: symbol) => void>('standards-mirror:unregister');
const activeId = inject<Ref<string | null>>('standards-mirror:active-id');
const activeTab = inject<Ref<'spec' | 'solidity' | 'csharp'>>('standards-mirror:active-tab');

const uid = Symbol(props.id);

const isActive = computed(() => activeId?.value === props.id);

onMounted(() => {
  register?.({
    uid,
    id: props.id,
    title: props.title,
    eip: props.eip,
    status: props.status,
    neoMapping: props.neoMapping,
    category: props.category,
    parityLabel: props.parityLabel,
    parityClass: props.parityClass
  });
});

onBeforeUnmount(() => {
  unregister?.(uid);
});
</script>

<template>
  <article v-show="isActive" class="standard-entry" :data-entry-id="id">
    <section v-show="activeTab === 'spec'" class="entry-section entry-spec">
      <slot name="spec" />
    </section>
    <section v-show="activeTab === 'solidity'" class="entry-section entry-solidity">
      <slot name="solidity" />
    </section>
    <section v-show="activeTab === 'csharp'" class="entry-section entry-csharp">
      <slot name="csharp" />
    </section>
  </article>
</template>

<style scoped>
.standard-entry {
  width: 100%;
}

.entry-section {
  width: 100%;
  min-width: 0;
}

.entry-section :deep(:first-child) {
  margin-top: 0 !important;
}

.entry-section :deep(:last-child) {
  margin-bottom: 0 !important;
}
</style>
