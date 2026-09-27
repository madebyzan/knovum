<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useCanvasSelectionProjection } from "../canvas/editor-session";
import { knowledgeLocationLabel } from "../knowledge/presentation/location-label";
import { useSpaceStore } from "../session/space-store";
import SpaceConversationPanel from "./conversation/SpaceConversationPanel.vue";
import SpaceDocumentOutline from "./editor/SpaceDocumentOutline.vue";
import SpaceSelectionInspector from "./inspector/SpaceSelectionInspector.vue";
import SpaceContextHeader from "./SpaceContextHeader.vue";
import SpaceSourcePeek from "./SpaceSourcePeek.vue";

const store = useSpaceStore();
const { rightSidebarView, sourcePeek } = storeToRefs(store);
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const { t } = useI18n();

watch(
  () => store.activeSpaceId,
  () => {
    if (store.sourcePeek) store.closeSource();
  },
);

watch(
  () => store.focusedEditorId,
  (id) => {
    if (id) return;
    if (store.rightSidebarView === "document-outline") store.setRightSidebarView("ask");
    if (store.sourceReturnView === "document-outline") store.sourceReturnView = "ask";
  },
);

const title = computed(() => {
  if (rightSidebarView.value === "source") return sourcePeek.value?.nodeTitle ?? t("spaces.inspector.source");
  if (selectedNodes.value.length === 1) return t("spaces.inspector.title");
  return selectedNodes.value.length
    ? t("spaces.inspector.itemsSelected", { count: selectedNodes.value.length })
    : t("spaces.knowledge.health.title");
});
const subtitle = computed(() =>
  rightSidebarView.value === "source" && sourcePeek.value
    ? knowledgeLocationLabel(sourcePeek.value.locator)
    : undefined,
);
</script>

<template>
  <aside
    :data-sidebar-view="rightSidebarView"
    class="pointer-events-auto flex h-full w-full min-w-0 flex-col overflow-hidden bg-panel select-none"
  >
    <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <SpaceContextHeader
        v-if="rightSidebarView === 'inspect' || rightSidebarView === 'source'"
        :title="title"
        v-bind="subtitle === undefined ? {} : { subtitle }"
        :back="rightSidebarView === 'source'"
        @back="store.closeSource()"
      />
      <div class="min-h-0 flex-1 overflow-hidden select-text">
        <SpaceSelectionInspector v-if="rightSidebarView === 'inspect'" class="h-full" />
        <SpaceConversationPanel v-else-if="rightSidebarView === 'ask'" class="h-full" />
        <SpaceDocumentOutline v-else-if="rightSidebarView === 'document-outline'" class="h-full" />
        <SpaceSourcePeek v-if="rightSidebarView === 'source' && sourcePeek" class="h-full" />
      </div>
    </section>
  </aside>
</template>
