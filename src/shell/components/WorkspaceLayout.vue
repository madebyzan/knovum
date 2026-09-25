<script setup lang="ts">
import { SplitterGroup, SplitterPanel } from "reka-ui";
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import UiSplitterHandle from "../../shared/ui/UiSplitterHandle.vue";
import FeatureErrorBoundary from "../FeatureErrorBoundary.vue";
import { workspaceSidebarGeometry } from "../sidebar-geometry";
import TaskCenter from "./TaskCenter.vue";
import WorkspaceRightSidebar from "./WorkspaceRightSidebar.vue";
import WorkspaceSidebar from "./WorkspaceSidebar.vue";

const props = defineProps<{
  workspace: WorkspaceDefinition;
  sidebarVisible: boolean;
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
  "open-sidebar-preview": [];
  "schedule-sidebar-close": [];
  "close-sidebar-preview": [];
}>();
const { t } = useI18n();
const motionReady = shallowRef(false);
const rightSidebarWidth = computed(() =>
  Math.min(
    props.workspace.rightSidebarLayout.maxSize,
    Math.max(props.workspace.rightSidebarLayout.minSize, props.rightSidebarWidth),
  ),
);

onMounted(() => {
  requestAnimationFrame(() => requestAnimationFrame(() => (motionReady.value = true)));
});

function updateSidebarWidth(size: number) {
  if (size === 0) {
    emit("set-sidebar-pinned", false);
    return;
  }
  if (size >= workspaceSidebarGeometry.minSize) {
    if (!props.sidebarPinned) emit("set-sidebar-pinned", true);
    emit("set-sidebar-width", size);
  }
}
</script>

<template>
  <div class="relative flex min-h-0 min-w-0 flex-1">
    <SplitterGroup
      direction="horizontal"
      class="workspace-splitter-group min-h-0 min-w-0 flex-1"
      :class="{ 'is-motion-ready': motionReady }"
    >
      <SplitterPanel
        v-if="sidebarPinned"
        id="spaces-sidebar"
        :order="1"
        size-unit="px"
        :default-size="sidebarWidth"
        :min-size="workspaceSidebarGeometry.minSize"
        :max-size="workspaceSidebarGeometry.maxSize"
        :collapsed-size="0"
        collapsible
        @resize="updateSidebarWidth"
      />
      <UiSplitterHandle v-if="sidebarPinned" />

      <SplitterPanel id="spaces-work-area" :order="2" :min-size="35">
        <main
          class="h-full min-w-0 transition-[padding-left] duration-200 ease-standard select-text"
          :class="[sidebarPinned ? 'pb-2' : 'pb-2 pl-2', rightSidebarOpen ? '' : 'pr-2']"
        >
          <div
            class="relative h-full min-h-0 overflow-hidden border border-window-hairline bg-canvas shadow-workspace"
            :class="rightSidebarOpen ? 'rounded-l-workspace border-r-0' : 'rounded-workspace'"
          >
            <FeatureErrorBoundary reset-key="spaces">
              <component :is="workspace.view" />
            </FeatureErrorBoundary>
            <div class="absolute right-3 bottom-3 z-30">
              <TaskCenter />
            </div>
          </div>
        </main>
      </SplitterPanel>

      <template v-if="rightSidebarOpen">
        <UiSplitterHandle />
        <SplitterPanel
          id="spaces-right-sidebar"
          :order="3"
          size-unit="px"
          :default-size="rightSidebarWidth"
          :min-size="workspace.rightSidebarLayout.minSize"
          :max-size="workspace.rightSidebarLayout.maxSize"
          @resize="emit('set-right-sidebar-width', $event)"
        >
          <div class="h-full min-h-0 pr-4 pb-2">
            <WorkspaceRightSidebar class="h-full min-h-0" :workspace="workspace" />
          </div>
        </SplitterPanel>
      </template>
    </SplitterGroup>

    <WorkspaceSidebar
      class="workspace-sidebar-surface absolute inset-y-0 left-0 z-40 h-full min-h-0"
      :class="{
        'is-visible': sidebarVisible,
        'is-preview rounded-r-workspace': sidebarPreview,
      }"
      :style="{ width: `${sidebarWidth}px` }"
      :aria-hidden="!sidebarVisible"
      :inert="!sidebarVisible"
      :workspace="workspace"
      @pointerenter="emit('open-sidebar-preview')"
      @pointerleave="emit('schedule-sidebar-close')"
    />

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
        class="absolute inset-0 z-30 bg-scrim"
        :aria-label="t('shell.toggleSidebar')"
        @click="emit('close-sidebar-preview')"
      />
    </Transition>
  </div>
</template>

<style scoped src="./workspace-layout.css"></style>
