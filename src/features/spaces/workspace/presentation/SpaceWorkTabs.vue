<script setup lang="ts">
import { FileText, Film, LayoutDashboard, X } from "@lucide/vue";
import { TabsList, TabsTrigger } from "reka-ui";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import { currentCanvasEditor } from "../../canvas/editor-session";
import { useDocumentSessionStore } from "../../document-editing/document-session-store";
import { useSpaceStore } from "../../session/space-store";
import { useVideoSessionStore } from "../../video-editing/video-session-store";
import type { SpaceEditorTab } from "../work-items";

const store = useSpaceStore();
const documents = useDocumentSessionStore();
const videos = useVideoSessionStore();
const { t } = useI18n();
const viewport = useTemplateRef<HTMLElement>("viewport");
const tabs = computed(() => store.editorTabs.filter((tab) => tab.spaceId === store.activeSpaceId));
const activeId = computed(() =>
  store.activeSpaceId ? store.activeWorkItems.get(store.activeSpaceId) : undefined,
);
function label(tab: SpaceEditorTab) {
  const document = documents.sessions.get(tab.id);
  return (
    document?.state.title.value ||
    document?.node.value.data.title ||
    videos.sessions.get(tab.id)?.node.value.data.title
  );
}
function dirty(tab: SpaceEditorTab) {
  return tab.kind === "document"
    ? documents.sessions.get(tab.id)?.dirty.value
    : videos.sessions.get(tab.id)?.history.dirty.value;
}
async function close(tabId: string) {
  try {
    await store.closeEditorTab(tabId);
  } catch (error) {
    if (!isApplicationLeaveCancelled(error)) toast.error(getErrorMessage(error));
  }
}
watch([() => store.editorTabs, activeId], async ([items], [, previousId]) => {
  if (!previousId || items.some(({ id }) => id === previousId)) return;
  await nextTick();
  const successor = viewport.value?.querySelector<HTMLElement>("[role=tab][data-state=active]");
  if (successor) successor.focus();
  else currentCanvasEditor()?.focus();
});
watch(activeId, async () => {
  await nextTick();
  viewport.value
    ?.querySelector<HTMLElement>("[role=tab][data-state=active]")
    ?.scrollIntoView({ block: "nearest", inline: "nearest" });
});
</script>

<template>
  <div
    v-if="tabs.length && store.surface.kind !== 'library'"
    ref="viewport"
    class="shrink-0 overflow-x-auto border-b border-stroke-divider bg-surface-chrome px-2"
  >
    <TabsList :aria-label="t('spaces.editor.tabs.label')" class="flex w-max items-center gap-1 py-1">
      <TabsTrigger
        value="canvas"
        data-work-tab="canvas"
        class="flex h-(--knovum-control-height-sm) items-center gap-1.5 rounded-md px-3 text-label text-fg-secondary hover:bg-surface-control-hover data-[state=active]:bg-surface-raised data-[state=active]:text-fg-primary"
      >
        <LayoutDashboard class="size-3.5" />{{ t("spaces.editor.tabs.canvas") }}
      </TabsTrigger>
      <div
        v-for="tab in tabs"
        :key="tab.id"
        :data-work-tab="tab.id"
        class="group flex max-w-60 items-center rounded-md"
        :class="activeId === tab.id && 'bg-surface-raised'"
        @auxclick.middle.prevent="close(tab.id)"
      >
        <TabsTrigger
          :value="tab.id"
          class="flex h-(--knovum-control-height-sm) min-w-0 items-center gap-1.5 px-3 text-label text-fg-secondary data-[state=active]:text-fg-primary"
        >
          <component :is="tab.kind === 'document' ? FileText : Film" class="size-3.5 shrink-0" />
          <span class="truncate">{{ label(tab) }}</span>
          <span v-if="dirty(tab)" aria-hidden="true">•</span>
          <span v-if="dirty(tab)" class="sr-only">{{ t("common.unsaved") }}</span>
        </TabsTrigger>
        <button
          type="button"
          class="mr-1 grid size-5 shrink-0 place-items-center rounded text-fg-secondary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
          :aria-label="t('spaces.editor.tabs.close', { title: label(tab) })"
          @click="close(tab.id)"
        >
          <X class="size-3" />
        </button>
      </div>
    </TabsList>
  </div>
</template>
