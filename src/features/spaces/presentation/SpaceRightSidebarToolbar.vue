<script setup lang="ts">
import { FileText, Info, MessageSquareText } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import { useContextDock } from "../canvas/use-context-dock";
import { useSpaceStore } from "../session/space-store";

const store = useSpaceStore();
const { rightSidebarView } = storeToRefs(store);
const { t } = useI18n();
const dock = useContextDock();
</script>

<template>
  <nav class="flex min-w-0 items-center gap-1" :aria-label="t('spaces.sidebar.rightSidebarViews')">
    <UiIconButton
      :label="t('spaces.assistant.title')"
      :active="
        rightSidebarView === 'ask' || (rightSidebarView === 'source' && store.sourceReturnView === 'ask')
      "
      size="md"
      @click="dock.openAsk()"
    >
      <MessageSquareText class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.55" />
    </UiIconButton>
    <UiIconButton
      :label="t('spaces.inspector.title')"
      :active="
        rightSidebarView === 'inspect' ||
        (rightSidebarView === 'source' && store.sourceReturnView === 'inspect')
      "
      size="md"
      @click="dock.openInspect()"
    >
      <Info class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.55" />
    </UiIconButton>
    <UiIconButton
      v-if="store.focusedEditorId"
      :label="t('spaces.sidebar.editorOutline.title')"
      :active="
        rightSidebarView === 'document-outline' ||
        (rightSidebarView === 'source' && store.sourceReturnView === 'document-outline')
      "
      size="md"
      @click="dock.openDocumentOutline()"
    >
      <FileText class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.55" />
    </UiIconButton>
  </nav>
</template>
