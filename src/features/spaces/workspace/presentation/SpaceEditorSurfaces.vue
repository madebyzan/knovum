<script setup lang="ts">
import { defineAsyncComponent, inject } from "vue";

import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";
import SpaceDocumentTab from "../../document-editing/presentation/SpaceDocumentTab.vue";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceEditorTab } from "../workbench-tab";

const store = useSpaceStore();
const visible = inject(overlayScopeVisibleKey)!;
const SpaceVideoEditor = defineAsyncComponent(
  () => import("../../video-editing/presentation/SpaceVideoEditor.vue"),
);

/** Open editors stay mounted so vendor instances survive tab switches; only the active one shows. */
function active(tab: SpaceEditorTab) {
  return visible.value && store.activeTabId === tab.id && store.surface.kind !== "library";
}
</script>

<template>
  <section
    v-for="tab in store.editorTabs"
    :key="tab.id"
    v-show="active(tab)"
    class="absolute inset-0 outline-none"
  >
    <SpaceDocumentTab v-if="tab.kind === 'document'" :tab="tab" :active="active(tab)" />
    <SpaceVideoEditor v-else :tab="tab" :active="active(tab)" />
  </section>
</template>
