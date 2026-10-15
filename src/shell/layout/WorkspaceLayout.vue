<script setup lang="ts">
import { SplitterGroup, SplitterPanel } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiSplitterHandle from "../../shared/ui/UiSplitterHandle.vue";
import FeatureErrorBoundary from "../FeatureErrorBoundary.vue";
import { WORKSPACE_SIDEBAR_GEOMETRY } from "./sidebar-geometry";
import { useWorkspaceSplitter } from "./use-workspace-splitter";
import WorkspaceRightSidebar from "./WorkspaceRightSidebar.vue";
import WorkspaceSidebar from "./WorkspaceSidebar.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  sidebarWidth: number;
  sidebarPreview: boolean;
  rightSidebarOpen: boolean;
  rightSidebarContentMounted: boolean;
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
const sidebarVisible = computed(() => props.sidebarPinned || props.sidebarPreview);
const dragging = shallowRef(false);
const liveSidebarWidth = shallowRef<number>();
/**
 * The surface is a floating host, so it cannot inherit the spacer panel's width. During a drag the committed
 * width is stale by design, and the surface must track the handle rather than jump on release.
 */
const surfaceWidth = computed(() =>
  dragging.value && liveSidebarWidth.value !== undefined ? liveSidebarWidth.value : props.sidebarWidth,
);
const { handleLayout, leadingPanel, trailingPanel } = useWorkspaceSplitter({
  sidebarPinned: () => props.sidebarPinned,
  sidebarWidth: () => props.sidebarWidth,
  rightSidebarOpen: () => props.rightSidebarOpen,
  rightSidebarWidth: () => props.rightSidebarWidth,
  onLayout: (sizes) => emit("panel-layout", sizes),
});
function onPanelLayout(sizes: number[]) {
  if (dragging.value) liveSidebarWidth.value = leadingPanel.value?.getSize();
  handleLayout(sizes);
}
function commitSidebarWidth() {
  const size = leadingPanel.value?.getSize();
  if (size === 0) emit("set-sidebar-pinned", false);
  else if (size && size >= WORKSPACE_SIDEBAR_GEOMETRY.minSize) emit("set-sidebar-width", size);
}
function commitRightSidebarWidth() {
  const size = trailingPanel.value?.getSize();
  if (size === 0) emit("close-right-sidebar");
  else if (size && size >= props.workspace.rightSidebarLayout.minSize) emit("set-right-sidebar-width", size);
}
function onDrag(active: boolean, commit: () => void) {
  dragging.value = active;
  if (active) {
    emit("cancel-panel-motion");
    liveSidebarWidth.value = leadingPanel.value?.getSize();
  } else commit();
}
/** Hover only governs the preview role; a pinned sidebar is already open and owns no hover state. */
function openPreview() {
  if (props.sidebarPinned) return;
  emit("open-sidebar-preview");
}
function schedulePreviewClose() {
  if (props.sidebarPinned) return;
  emit("schedule-sidebar-close");
}
</script>

<template>
  <div class="relative flex min-h-0 min-w-0 flex-1">
    <!--
      One host serves both sidebar roles, and it precedes the splitter so focus order still starts at the
      sidebar. Pinning changes only whether the splitter reserves width for it. Moving the subtree between
      hosts instead would reset its scroll position and make every outline row report a new height through
      its resize observer, all inside the first frame of the geometry transition.
    -->
    <div
      class="workspace-sidebar-surface absolute inset-y-0 left-0 z-40 h-full min-h-0 overflow-hidden"
      :class="{ 'is-visible': sidebarVisible, 'is-preview rounded-r-xl': sidebarPreview && !sidebarPinned }"
      :style="{ width: `${surfaceWidth}px` }"
      :aria-hidden="!sidebarVisible"
      :inert="!sidebarVisible"
      @pointerenter="openPreview"
      @pointerleave="schedulePreviewClose"
    >
      <WorkspaceSidebar :workspace="workspace" />
    </div>
    <SplitterGroup
      direction="horizontal"
      class="workspace-splitter-group relative z-10 min-h-0 min-w-0 flex-1"
      @layout="onPanelLayout"
    >
      <SplitterPanel
        ref="leadingPanel"
        id="spaces-sidebar"
        :order="1"
        size-unit="px"
        :default-size="sidebarPinned ? sidebarWidth : 0"
        :min-size="WORKSPACE_SIDEBAR_GEOMETRY.minSize"
        :max-size="WORKSPACE_SIDEBAR_GEOMETRY.maxSize"
        :collapsed-size="0"
        collapsible
      />
      <UiSplitterHandle
        :disabled="!sidebarPinned"
        @dragging="onDrag($event, commitSidebarWidth)"
        @keyup="commitSidebarWidth"
      />

      <SplitterPanel id="spaces-work-area" :order="2" :min-size="35">
        <main class="h-full min-w-0 pb-2 select-text" :class="sidebarPinned ? 'pr-2 pl-0' : 'px-2'">
          <div
            class="relative h-full min-h-0 overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-canvas shadow-workspace"
          >
            <FeatureErrorBoundary reset-key="spaces">
              <component :is="workspace.view" />
            </FeatureErrorBoundary>
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
          class="h-full min-w-0 pr-2 pb-2"
          :style="{ '--workspace-right-sidebar-width': `${rightSidebarWidth}px` }"
          :inert="!rightSidebarOpen"
          :aria-hidden="!rightSidebarOpen"
        >
          <Transition name="workspace-pane-content">
            <WorkspaceRightSidebar v-if="rightSidebarContentMounted" :workspace="workspace" />
          </Transition>
        </div>
      </SplitterPanel>
    </SplitterGroup>

    <div
      v-if="!sidebarPinned && !sidebarPreview"
      class="absolute inset-y-0 left-0 z-50 w-3"
      @pointerenter="openPreview"
      @pointermove="openPreview"
      @pointerleave="schedulePreviewClose"
    />
    <Transition name="workspace-scrim">
      <button
        v-if="sidebarPreview"
        type="button"
        class="absolute inset-0 z-30 bg-surface-scrim"
        :aria-label="t('shell.layout.toggleSidebar')"
        @click="emit('close-sidebar-preview')"
      />
    </Transition>
  </div>
</template>

<style scoped src="./workspace-layout.css"></style>
