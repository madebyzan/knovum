<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import FeatureErrorBoundary from "../FeatureErrorBoundary.vue";
import { WORKSPACE_SIDEBAR_GEOMETRY } from "./sidebar-geometry";
import type { WorkspaceEdges } from "./use-workspace-edges";
import WorkspaceRightSidebar from "./WorkspaceRightSidebar.vue";
import WorkspaceSidebar from "./WorkspaceSidebar.vue";
import WorkspaceSidebarSash from "./WorkspaceSidebarSash.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarPinned: boolean;
  sidebarWidth: number;
  sidebarPreview: boolean;
  rightSidebarOpen: boolean;
  rightSidebarContentMounted: boolean;
  rightSidebarWidth: number;
  edges: WorkspaceEdges;
}>();
const emit = defineEmits<{
  "set-sidebar-width": [value: number];
  "set-right-sidebar-width": [value: number];
  "commit-layout": [];
  "open-sidebar-preview": [];
  "schedule-sidebar-close": [];
  "close-sidebar-preview": [];
}>();
const { t } = useI18n();
const sidebarVisible = computed(() => props.sidebarPinned || props.sidebarPreview);
const sidebarFloating = computed(() => props.sidebarPreview && !props.sidebarPinned);
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
  <div
    class="relative flex min-h-0 min-w-0 flex-1 overflow-hidden"
    :style="{
      '--workspace-sidebar-width': `${sidebarWidth}px`,
      '--workspace-right-sidebar-width': `${rightSidebarWidth}px`,
    }"
  >
    <!--
      The slot is only the layout width the sidebar reserves. The sidebar is a fixed-width surface placed by
      transform at its own edge, so it never reflows and moves on the compositor. Both read the same frame
      of the panel clock: pinning from a preview grows the slot under a sidebar that holds still, and closing
      keeps the sidebar's edge on the work area's edge in every frame. One surface serves pinned and preview:
      moving the subtree between hosts would reset its scroll and make every outline row re-measure.
    -->
    <div class="workspace-sidebar-slot" :style="{ '--workspace-leading-edge': `${edges.leading}px` }" />
    <div
      id="spaces-sidebar"
      class="workspace-sidebar-surface"
      :class="{ 'is-floating': sidebarFloating, 'rounded-r-xl': !sidebarPinned }"
      :style="{ transform: `translate3d(${edges.sidebar - sidebarWidth}px, 0, 0)` }"
      :aria-hidden="!sidebarVisible"
      :inert="!sidebarVisible"
      @pointerenter="openPreview"
      @pointerleave="schedulePreviewClose"
    >
      <div class="workspace-sidebar-elevation rounded-[inherit]" aria-hidden="true" />
      <div class="relative h-full min-h-0 overflow-hidden rounded-[inherit]">
        <WorkspaceSidebar :workspace="workspace" />
      </div>
    </div>

    <WorkspaceSidebarSash
      v-if="sidebarPinned"
      edge="leading"
      controls="spaces-sidebar"
      :label="t('shell.layout.resizeSidebar')"
      :width="sidebarWidth"
      :geometry="WORKSPACE_SIDEBAR_GEOMETRY"
      @resize="emit('set-sidebar-width', $event)"
      @commit="emit('commit-layout')"
    />

    <main
      class="min-h-0 min-w-0 flex-1 pr-(--knovum-workspace-gutter) pb-(--knovum-workspace-gutter) select-text"
    >
      <div
        class="relative h-full min-h-0 overflow-hidden rounded-xl border border-(--knovum-window-border-color) bg-surface-canvas shadow-workspace"
      >
        <FeatureErrorBoundary reset-key="spaces">
          <component :is="workspace.view" />
        </FeatureErrorBoundary>
      </div>
    </main>

    <WorkspaceSidebarSash
      v-if="rightSidebarOpen"
      edge="trailing"
      controls="spaces-right-sidebar"
      :label="t('shell.layout.resizeRightSidebar')"
      :width="rightSidebarWidth"
      :geometry="workspace.rightSidebarLayout"
      @resize="emit('set-right-sidebar-width', $event)"
      @commit="emit('commit-layout')"
    />
    <div
      id="spaces-right-sidebar"
      class="workspace-right-sidebar-slot"
      :style="{ width: `${edges.trailing}px` }"
      :inert="!rightSidebarOpen"
      :aria-hidden="!rightSidebarOpen"
    >
      <Transition name="workspace-pane-content">
        <WorkspaceRightSidebar v-if="rightSidebarContentMounted" :workspace="workspace" />
      </Transition>
    </div>

    <div
      v-if="!sidebarVisible"
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
