<script setup lang="ts">
import { SplitterGroup, SplitterPanel } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
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
const leadingPanel = shallowRef<InstanceType<typeof SplitterPanel> | null>(null);
const trailingPanel = shallowRef<InstanceType<typeof SplitterPanel> | null>(null);
const layoutReady = shallowRef(false);
const motionReady = shallowRef(false);
const rightWidth = computed(() =>
  Math.min(
    props.workspace.rightSidebarLayout.maxSize,
    Math.max(props.workspace.rightSidebarLayout.minSize, props.rightSidebarWidth),
  ),
);

watch(
  () => [props.sidebarPinned, layoutReady.value] as const,
  ([pinned, ready]) => {
    if (!ready) return;
    if (pinned) leadingPanel.value?.resize(props.sidebarWidth);
    else leadingPanel.value?.collapse();
  },
  { flush: "post" },
);
watch(
  () => [props.rightSidebarOpen, layoutReady.value] as const,
  ([open, ready]) => {
    if (!ready) return;
    if (open) trailingPanel.value?.resize(rightWidth.value);
    else trailingPanel.value?.collapse();
  },
  { flush: "post" },
);
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

function markLayoutReady(sizes: number[]) {
  if (sizes.length !== 3 || layoutReady.value) return;
  layoutReady.value = true;
  requestAnimationFrame(() => requestAnimationFrame(() => (motionReady.value = true)));
}
</script>

<template>
  <div class="relative flex min-h-0 min-w-0 flex-1">
    <div
      class="workspace-frame pointer-events-none absolute top-0 right-2 bottom-2 z-20 rounded-workspace border border-window-hairline shadow-workspace"
      :style="{ left: sidebarPinned ? `${sidebarWidth}px` : '8px' }"
    />
    <SplitterGroup
      direction="horizontal"
      class="workspace-splitter-group relative z-10 min-h-0 min-w-0 flex-1"
      :class="{ 'is-motion-ready': motionReady }"
      @layout="markLayoutReady"
    >
      <SplitterPanel
        ref="leadingPanel"
        id="spaces-sidebar"
        :order="1"
        size-unit="px"
        :default-size="sidebarPinned ? sidebarWidth : 0"
        :min-size="workspaceSidebarGeometry.minSize"
        :max-size="workspaceSidebarGeometry.maxSize"
        :collapsed-size="0"
        collapsible
        @resize="updateSidebarWidth"
      />
      <UiSplitterHandle :disabled="!sidebarPinned" />

      <SplitterPanel id="spaces-work-area" :order="2" :min-size="35">
        <main
          class="h-full min-w-0 transition-[padding-left,padding-right] duration-200 ease-standard select-text"
          :class="[sidebarPinned ? 'pb-2' : 'pb-2 pl-2', rightSidebarOpen ? '' : 'pr-2']"
        >
          <div
            class="relative h-full min-h-0 overflow-hidden bg-canvas"
            :class="rightSidebarOpen ? 'rounded-l-workspace' : 'rounded-workspace'"
          >
            <FeatureErrorBoundary reset-key="spaces">
              <component :is="workspace.view" />
            </FeatureErrorBoundary>
            <div class="absolute right-3 bottom-3 z-30"><TaskCenter /></div>
          </div>
        </main>
      </SplitterPanel>

      <UiSplitterHandle :disabled="!rightSidebarOpen" />
      <SplitterPanel
        ref="trailingPanel"
        id="spaces-right-sidebar"
        :order="3"
        size-unit="px"
        :default-size="rightSidebarOpen ? rightWidth : 0"
        :min-size="workspace.rightSidebarLayout.minSize"
        :max-size="workspace.rightSidebarLayout.maxSize"
        :collapsed-size="0"
        collapsible
        @resize="($event) => $event > 0 && emit('set-right-sidebar-width', $event)"
      >
        <div
          class="h-full min-h-0 pr-2 pb-2"
          :style="{ width: `${rightWidth}px` }"
          :inert="!rightSidebarOpen"
          :aria-hidden="!rightSidebarOpen"
        >
          <div
            class="h-full min-h-0 overflow-hidden rounded-r-workspace"
            :class="{ 'border-l border-window-hairline': rightSidebarOpen }"
          >
            <Transition name="workspace-pane-content">
              <WorkspaceRightSidebar v-if="rightSidebarOpen" class="h-full min-h-0" :workspace="workspace" />
            </Transition>
          </div>
        </div>
      </SplitterPanel>
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
