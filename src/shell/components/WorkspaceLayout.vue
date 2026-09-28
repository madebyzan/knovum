<script setup lang="ts">
import { SplitterGroup, SplitterPanel } from "reka-ui";
import { useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiSplitterHandle from "../../shared/ui/UiSplitterHandle.vue";
import FeatureErrorBoundary from "../FeatureErrorBoundary.vue";
import { workspaceSidebarGeometry } from "../sidebar-geometry";
import { useWorkspaceSplitter } from "../use-workspace-splitter";
import TaskCenter from "./TaskCenter.vue";
import WorkspaceRightSidebar from "./WorkspaceRightSidebar.vue";
import WorkspaceSidebar from "./WorkspaceSidebar.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  sidebarWidth: number;
  sidebarPreview: boolean;
  rightSidebarOpen: boolean;
  rightSidebarWidth: number;
}>();
const emit = defineEmits<{
  "set-sidebar-pinned": [value: boolean];
  "set-sidebar-width": [value: number];
  "set-right-sidebar-width": [value: number];
  "close-right-sidebar": [];
  "panel-layout": [sizes: number[]];
  "cancel-panel-motion": [];
  "open-sidebar-preview": [];
  "schedule-sidebar-close": [];
  "close-sidebar-preview": [];
}>();
const { t } = useI18n();
const pinnedSidebarHost = useTemplateRef<HTMLElement>("pinnedSidebarHost");
const previewSidebarHost = useTemplateRef<HTMLElement>("previewSidebarHost");
const { handleLayout, leadingPanel, trailingPanel } = useWorkspaceSplitter({
  sidebarPinned: () => props.sidebarPinned,
  sidebarWidth: () => props.sidebarWidth,
  rightSidebarOpen: () => props.rightSidebarOpen,
  rightSidebarWidth: () => props.rightSidebarWidth,
  onLayout: (sizes) => emit("panel-layout", sizes),
});
function commitSidebarWidth() {
  const size = leadingPanel.value?.getSize();
  if (size === 0) emit("set-sidebar-pinned", false);
  else if (size && size >= workspaceSidebarGeometry.minSize) emit("set-sidebar-width", size);
}
function commitRightSidebarWidth() {
  const size = trailingPanel.value?.getSize();
  if (size === 0) emit("close-right-sidebar");
  else if (size && size >= props.workspace.rightSidebarLayout.minSize)
    emit("set-right-sidebar-width", size);
}
function onDrag(active: boolean, commit: () => void) {
  if (active) emit("cancel-panel-motion");
  else commit();
}
</script>

<template>
  <div class="relative flex min-h-0 min-w-0 flex-1">
    <SplitterGroup
      direction="horizontal"
      class="workspace-splitter-group relative z-10 min-h-0 min-w-0 flex-1"
      @layout="handleLayout"
    >
      <SplitterPanel
        ref="leadingPanel"
        id="spaces-sidebar"
        class="overflow-hidden"
        :order="1"
        size-unit="px"
        :default-size="sidebarPinned ? sidebarWidth : 0"
        :min-size="workspaceSidebarGeometry.minSize"
        :max-size="workspaceSidebarGeometry.maxSize"
        :collapsed-size="0"
        collapsible
      >
        <div ref="pinnedSidebarHost" class="h-full w-full" :style="{ minWidth: `${sidebarWidth}px` }" />
      </SplitterPanel>
      <UiSplitterHandle
        :disabled="!sidebarPinned"
        @dragging="onDrag($event, commitSidebarWidth)"
        @keyup="commitSidebarWidth"
      />

      <SplitterPanel id="spaces-work-area" :order="2" :min-size="35">
        <main class="h-full min-w-0 px-2 pb-2 select-text">
          <div
            class="relative h-full min-h-0 overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-canvas shadow-workspace"
          >
            <FeatureErrorBoundary reset-key="spaces">
              <component :is="workspace.view" />
            </FeatureErrorBoundary>
            <div class="absolute right-3 bottom-3 z-30"><TaskCenter /></div>
          </div>
        </main>
      </SplitterPanel>

      <UiSplitterHandle
        :disabled="!rightSidebarOpen"
        @dragging="onDrag($event, commitRightSidebarWidth)"
        @keyup="commitRightSidebarWidth"
      />
      <SplitterPanel
        ref="trailingPanel"
        id="spaces-right-sidebar"
        class="relative overflow-hidden"
        :order="3"
        size-unit="px"
        :default-size="rightSidebarOpen ? rightSidebarWidth : 0"
        :min-size="workspace.rightSidebarLayout.minSize"
        :max-size="workspace.rightSidebarLayout.maxSize"
        :collapsed-size="0"
        collapsible
      >
        <div
          class="h-full min-w-0 pb-2 pr-2"
          :style="{ '--workspace-right-sidebar-width': `${rightSidebarWidth}px` }"
          :inert="!rightSidebarOpen"
          :aria-hidden="!rightSidebarOpen"
        >
          <Transition name="workspace-pane-content">
            <WorkspaceRightSidebar v-if="rightSidebarOpen" :workspace="workspace" />
          </Transition>
        </div>
      </SplitterPanel>
    </SplitterGroup>

    <div
      ref="previewSidebarHost"
      class="workspace-sidebar-surface absolute inset-y-0 left-0 z-40 h-full min-h-0"
      :class="sidebarPreview ? 'is-visible is-preview rounded-r-xl' : ''"
      :style="{ width: `${sidebarWidth}px` }"
      :aria-hidden="!sidebarPreview"
      :inert="!sidebarPreview"
      @pointerenter="emit('open-sidebar-preview')"
      @pointerleave="emit('schedule-sidebar-close')"
    />
    <Teleport
      v-if="pinnedSidebarHost && previewSidebarHost"
      :to="sidebarPinned ? pinnedSidebarHost : previewSidebarHost"
    >
      <WorkspaceSidebar :workspace="workspace" />
    </Teleport>
    <div
      v-if="!sidebarPinned && !sidebarPreview"
      class="absolute inset-y-0 left-0 z-50 w-3"
      @pointerenter="emit('open-sidebar-preview')"
      @pointermove="emit('open-sidebar-preview')"
      @pointerleave="emit('schedule-sidebar-close')"
    />
    <Transition name="workspace-scrim">
      <button
        v-if="sidebarPreview"
        type="button"
        class="absolute inset-0 z-30 bg-surface-scrim"
        :aria-label="t('shell.toggleSidebar')"
        @click="emit('close-sidebar-preview')"
      />
    </Transition>
  </div>
</template>

<style scoped src="./workspace-layout.css"></style>
