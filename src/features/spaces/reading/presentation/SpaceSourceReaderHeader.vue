<script setup lang="ts">
import {
  BookOpen,
  ChevronLeft,
  ChevronRight,
  LayoutList,
  Maximize2,
  Minimize2,
  MoreHorizontal,
  X,
} from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSegmentedControl from "../../../../shared/ui/UiSegmentedControl.vue";
import { nodeIcon } from "../../canvas/presentation/node-icon";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SourceReaderHost, SourceReaderPane } from "../source-reader-target";
import SpaceReaderTitleInput from "./SpaceReaderTitleInput.vue";
import { useReaderHeaderMenu } from "./use-reader-header-menu";

const props = defineProps<{
  node: SpaceCanvasNode;
  host: SourceReaderHost;
  pane: SourceReaderPane;
  readable: boolean;
  canBack: boolean;
  canForward: boolean;
  followsSelection: boolean;
  focusMode?: boolean | undefined;
  locator?: KnowledgeLocator | undefined;
}>();

const menuOpen = defineModel<boolean>("menuOpen", { default: false });

const emit = defineEmits<{
  "update:pane": [pane: SourceReaderPane];
  stepHistory: [delta: -1 | 1];
  toggleFollow: [];
  toggleFocusMode: [];
  close: [];
}>();

const { t } = useI18n();
const paneOptions = computed(() => [
  { value: "read" as const, title: t("spaces.inspector.reader.read"), icon: BookOpen },
  { value: "info" as const, title: t("spaces.inspector.title"), icon: LayoutList },
]);
const menuItems = useReaderHeaderMenu({
  node: () => props.node,
  followsSelection: () => props.followsSelection,
  locator: () => props.locator,
  onToggleFollow: () => emit("toggleFollow"),
});
</script>

<template>
  <header
    data-reader-ignore-selection
    class="flex h-(--knovum-titlebar-height) shrink-0 items-center justify-between gap-2 border-b border-(--knovum-window-border-color) bg-surface-panel px-3 select-none"
  >
    <div class="flex min-w-0 flex-1 items-center gap-1">
      <div v-if="canBack || canForward" class="mr-0.5 flex shrink-0 items-center">
        <UiIconButton
          :label="t('spaces.space.back')"
          size="sm"
          tooltip
          :disabled="!canBack"
          @click="emit('stepHistory', -1)"
        >
          <ChevronLeft class="size-3.5" :stroke-width="1.65" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.space.forward')"
          size="sm"
          tooltip
          :disabled="!canForward"
          @click="emit('stepHistory', 1)"
        >
          <ChevronRight class="size-3.5" :stroke-width="1.65" />
        </UiIconButton>
      </div>
      <component
        :is="nodeIcon(node.data.kind)"
        class="size-3.5 shrink-0 text-fg-tertiary"
        :stroke-width="1.6"
      />
      <SpaceReaderTitleInput v-if="host === 'canvas'" :node="node" />
      <h2
        v-else
        :title="node.data.title"
        class="min-w-0 truncate px-1 text-label font-semibold tracking-title text-fg-primary"
      >
        {{ node.data.title }}
      </h2>
    </div>

    <div class="flex shrink-0 items-center gap-1">
      <template v-if="host === 'canvas'">
        <UiSegmentedControl
          v-if="readable"
          :model-value="pane"
          :options="paneOptions"
          icon-only
          @update:model-value="emit('update:pane', $event)"
        />
        <UiIconButton
          :label="
            t(focusMode ? 'spaces.inspector.reader.exitFocusMode' : 'spaces.inspector.reader.focusMode')
          "
          :active="Boolean(focusMode)"
          size="sm"
          tooltip
          @click="emit('toggleFocusMode')"
        >
          <Minimize2 v-if="focusMode" class="size-3.5" :stroke-width="1.6" />
          <Maximize2 v-else class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiDropdownMenu v-model:open="menuOpen" :items="menuItems" align="end" class="min-w-44">
          <template #trigger>
            <UiIconButton :label="t('common.more')" size="sm" tooltip>
              <MoreHorizontal class="size-3.5" :stroke-width="1.6" />
            </UiIconButton>
          </template>
        </UiDropdownMenu>
      </template>
      <UiIconButton :label="t('common.close')" size="sm" tooltip @click="emit('close')">
        <X class="size-3.5" :stroke-width="1.6" />
      </UiIconButton>
    </div>
  </header>
</template>
