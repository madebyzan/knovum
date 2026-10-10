<script setup lang="ts">
import { TabsContent } from "reka-ui";
import { computed, inject, provide } from "vue";

import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";

const props = defineProps<{ value: string; active: string }>();
const parentVisibility = inject(overlayScopeVisibleKey, undefined);
const isVisible = computed(() => props.value === props.active && parentVisibility?.value !== false);
provide(overlayScopeVisibleKey, isVisible);
</script>

<template>
  <TabsContent :value="value" force-mount v-show="value === active" class="space-y-3 outline-none">
    <slot />
  </TabsContent>
</template>
