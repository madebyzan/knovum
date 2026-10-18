<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import SpaceConversation from "../../conversation/presentation/SpaceConversation.vue";
import SpaceSelectionInspector from "../../inspection/presentation/SpaceSelectionInspector.vue";
import SpaceSourcePeek from "../../knowledge/presentation/SpaceSourcePeek.vue";
import { useSpaceStore } from "../../session/space-store";
import SpaceContextHeader from "./SpaceContextHeader.vue";
import SpaceRightSidebarViewPicker from "./SpaceRightSidebarViewPicker.vue";

const store = useSpaceStore();
const { rightSidebarView, sourcePeek } = storeToRefs(store);
const { t } = useI18n();

watch(
  () => store.activeSpaceId,
  () => {
    if (store.sourcePeek) store.closeSource();
  },
);

const sourceTitle = computed(() => t("spaces.conversation.evidence.sourceRationale"));
</script>

<template>
  <aside
    :data-sidebar-view="rightSidebarView"
    class="pointer-events-auto flex h-full w-full min-w-0 flex-col overflow-hidden bg-surface-panel select-none"
  >
    <SpaceContextHeader
      v-if="rightSidebarView === 'source'"
      :title="sourceTitle"
      back
      closable
      @back="store.closeSource()"
      @close="store.closeSource()"
    />
    <SpaceContextHeader v-else-if="rightSidebarView === 'inspect'">
      <template #leading><SpaceRightSidebarViewPicker /></template>
    </SpaceContextHeader>
    <div class="min-h-0 flex-1 overflow-hidden select-text">
      <SpaceSelectionInspector v-if="rightSidebarView === 'inspect'" class="h-full" />
      <SpaceConversation v-else-if="rightSidebarView === 'ask'" class="h-full" />
      <SpaceSourcePeek v-if="rightSidebarView === 'source' && sourcePeek" class="h-full" />
    </div>
  </aside>
</template>
