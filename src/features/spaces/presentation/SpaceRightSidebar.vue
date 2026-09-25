<script setup lang="ts">
import { FileText, Info, MessageSquareText } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import { useCanvasSelectionProjection } from "../canvas/editor-session";
import { useContextDock } from "../canvas/use-context-dock";
import { knowledgeLocationLabel } from "../knowledge/model/location";
import { useSpaceStore } from "../session/space-store";
import SpaceConversationPanel from "./conversation/SpaceConversationPanel.vue";
import SpaceDocumentOutline from "./editor/SpaceDocumentOutline.vue";
import SpaceContextHeader from "./SpaceContextHeader.vue";
import SpaceInspectorDetails from "./SpaceInspectorDetails.vue";
import SpaceSourcePeek from "./SpaceSourcePeek.vue";

const store = useSpaceStore();
const { rightSidebarView, sourcePeek } = storeToRefs(store);
const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
const { t } = useI18n();
const dock = useContextDock();

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
  if (selectedNodes.value.length === 1) return t("spaces.navigation.details");
  return selectedNodes.value.length
    ? t("spaces.inspector.itemsSelected", { count: selectedNodes.value.length })
    : t("spaces.knowledge.knowledgeHealth");
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
    class="pointer-events-auto flex h-full w-full min-w-0 flex-col overflow-hidden rounded-r-workspace border-y border-r border-l border-window-hairline bg-panel select-none"
  >
    <nav
      class="flex h-9 shrink-0 items-center gap-0.5 border-b border-divider px-2"
      :aria-label="t('spaces.navigation.rightSidebarViews')"
    >
      <UiIconButton
        :label="t('spaces.assistant.title')"
        :active="
          rightSidebarView === 'ask' || (rightSidebarView === 'source' && store.sourceReturnView === 'ask')
        "
        size="sm"
        @click="dock.openAsk()"
      >
        <MessageSquareText class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.navigation.details')"
        :active="
          rightSidebarView === 'inspect' ||
          (rightSidebarView === 'source' && store.sourceReturnView === 'inspect')
        "
        size="sm"
        @click="dock.openInspect()"
      >
        <Info class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
      <UiIconButton
        v-if="store.focusedEditorId"
        :label="t('spaces.navigation.editorOutline')"
        :active="
          rightSidebarView === 'document-outline' ||
          (rightSidebarView === 'source' && store.sourceReturnView === 'document-outline')
        "
        size="sm"
        @click="dock.openDocumentOutline()"
      >
        <FileText class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </nav>
    <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
      <SpaceContextHeader
        v-if="rightSidebarView === 'inspect' || rightSidebarView === 'source'"
        :title="title"
        v-bind="subtitle === undefined ? {} : { subtitle }"
        :back="rightSidebarView === 'source'"
        @back="store.closeSource()"
        @close="dock.closeDock()"
      />
      <div class="min-h-0 flex-1 overflow-hidden select-text">
        <SpaceInspectorDetails v-if="rightSidebarView === 'inspect'" class="h-full" />
        <SpaceConversationPanel
          v-else-if="rightSidebarView === 'ask'"
          class="h-full"
          @close="dock.closeDock()"
        />
        <SpaceDocumentOutline
          v-else-if="rightSidebarView === 'document-outline'"
          class="h-full"
          @close="dock.closeDock()"
        />
        <SpaceSourcePeek v-if="rightSidebarView === 'source' && sourcePeek" class="h-full" />
      </div>
    </section>
  </aside>
</template>
