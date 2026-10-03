<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../session/space-store";
import SpaceSpeechPanel from "../speech/SpaceSpeechPanel.vue";
import SpaceConversationPanel from "./conversation/SpaceConversationPanel.vue";
import SpaceDocumentOutline from "./editor/SpaceDocumentOutline.vue";
import SpaceSelectionInspector from "./inspector/SpaceSelectionInspector.vue";
import { knowledgeLocationLabel } from "./knowledge/knowledge-location-label";
import SpaceContextHeader from "./SpaceContextHeader.vue";
import SpaceRightSidebarViewPicker from "./SpaceRightSidebarViewPicker.vue";
import SpaceSourcePeek from "./SpaceSourcePeek.vue";

const store = useSpaceStore();
const { rightSidebarView, sourcePeek } = storeToRefs(store);
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
  if (rightSidebarView.value === "speech") return t("spaces.speech.title");
  return undefined;
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
    class="pointer-events-auto flex h-full w-full min-w-0 flex-col overflow-hidden bg-surface-panel select-none"
  >
    <SpaceContextHeader
      v-if="rightSidebarView === 'inspect' || rightSidebarView === 'speech' || rightSidebarView === 'source'"
      :title="title"
      :subtitle="subtitle"
      :back="rightSidebarView === 'source'"
      @back="store.closeSource()"
    >
      <template #leading><SpaceRightSidebarViewPicker /></template>
    </SpaceContextHeader>
    <div class="min-h-0 flex-1 overflow-hidden select-text">
      <SpaceSelectionInspector v-if="rightSidebarView === 'inspect'" class="h-full" />
      <SpaceConversationPanel v-else-if="rightSidebarView === 'ask'" class="h-full" />
      <SpaceDocumentOutline v-else-if="rightSidebarView === 'document-outline'" class="h-full" />
      <SpaceSpeechPanel v-else-if="rightSidebarView === 'speech'" class="h-full" />
      <SpaceSourcePeek v-if="rightSidebarView === 'source' && sourcePeek" class="h-full" />
    </div>
  </aside>
</template>
