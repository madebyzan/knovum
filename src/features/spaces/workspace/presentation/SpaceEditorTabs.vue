<script setup lang="ts">
import { TabsContent } from "reka-ui";
import { computed, defineAsyncComponent, inject } from "vue";

import { overlayScopeVisibleKey } from "../../../../shared/contracts/overlay-scope-visibility";
import SpaceDocumentTab from "../../document-editing/presentation/SpaceDocumentTab.vue";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceEditorTab } from "../work-items";

const store = useSpaceStore();
const visible = inject(overlayScopeVisibleKey)!;
const activeId = computed(() =>
  store.activeSpaceId ? store.activeWorkItems.get(store.activeSpaceId) : undefined,
);
const SpaceVideoEditor = defineAsyncComponent(
  () => import("../../video-editing/presentation/SpaceVideoEditor.vue"),
);
function active(tab: SpaceEditorTab) {
  return visible.value && store.surface.kind !== "library" && activeId.value === tab.id;
}
</script>

<template>
  <TabsContent
    v-for="tab in store.editorTabs"
    :key="tab.id"
    :value="tab.id"
    force-mount
    v-show="active(tab)"
    class="absolute inset-0 outline-none"
  >
    <SpaceDocumentTab v-if="tab.kind === 'document'" :tab="tab" :active="active(tab)" />
    <SpaceVideoEditor v-else :tab="tab" :active="active(tab)" />
  </TabsContent>
</template>
