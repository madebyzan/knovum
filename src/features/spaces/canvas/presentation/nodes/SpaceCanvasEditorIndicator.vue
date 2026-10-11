<script setup lang="ts">
import { PencilLine } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../../session/space-store";

const props = defineProps<{ nodeId: string }>();
const store = useSpaceStore();
const { t } = useI18n();
const opened = computed(() =>
  store.editorTabs.some((tab) => tab.spaceId === store.activeSpaceId && tab.nodeId === props.nodeId),
);
</script>

<template>
  <span
    v-if="opened"
    role="status"
    :title="t('spaces.editor.tabs.opened')"
    :aria-label="t('spaces.editor.tabs.opened')"
  >
    <PencilLine class="size-3.5 text-fg-accent" :stroke-width="1.6" />
  </span>
</template>
