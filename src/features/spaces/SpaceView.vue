<script setup lang="ts">
import { AlertTriangle } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { TabsContent, TabsRoot } from "reka-ui";
import {
  computed,
  defineAsyncComponent,
  inject,
  nextTick,
  onBeforeUnmount,
  onMounted,
  provide,
  shallowRef,
} from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { normalizeError } from "../../foundation/errors/app-error";
import { getErrorMessage } from "../../foundation/errors/error-format";
import { registerWindowCloseCommit } from "../../foundation/lifecycle/window-close-coordinator";
import { isApplicationLeaveCancelled } from "../../shared/contracts/application-leave";
import { overlayScopeVisibleKey } from "../../shared/contracts/overlay-scope-visibility";
import UiButton from "../../shared/ui/UiButton.vue";
import { selectAndRevealSpaceNode } from "./canvas/selection-actions";
import { useContextDock } from "./canvas/use-context-dock";
import {
  registerConversationRunNavigation,
  restoreConversationRuns,
} from "./conversation/conversation-run-observation";
import { useConversationSessionStore } from "./conversation/conversation-session-store";
import { enrichmentProjectionKey } from "./enrichment/enrichment-projection-key";
import { useEnrichmentTaskProjection } from "./enrichment/use-enrichment-task-projection";
import SpaceCanvas from "./presentation/canvas/SpaceCanvas.vue";
import SpaceImageEditLeaveDialog from "./presentation/canvas/SpaceImageEditLeaveDialog.vue";
import SpaceLibrary from "./presentation/library/SpaceLibrary.vue";
import SpaceAudioRecordingDock from "./presentation/media/SpaceAudioRecordingDock.vue";
import SpaceEditorTabs from "./presentation/workspace/SpaceEditorTabs.vue";
import SpaceWorkTabs from "./presentation/workspace/SpaceWorkTabs.vue";
import { useSpaceStore } from "./session/space-store";

const SpaceRecentlyDeleted = defineAsyncComponent(
  () => import("./presentation/deletion/SpaceRecentlyDeleted.vue"),
);
const store = useSpaceStore();
const conversationSession = useConversationSessionStore();
const workspaceVisible = inject(overlayScopeVisibleKey)!;
const canvasVisible = computed(() => workspaceVisible.value && store.workbench.canvas);
const { ready, surface } = storeToRefs(store);
const activeWorkItem = computed(() =>
  surface.value.kind === "library"
    ? "library"
    : store.activeSpaceId
      ? (store.activeWorkItems.get(store.activeSpaceId) ?? "canvas")
      : "canvas",
);
function activateWorkItem(value: unknown) {
  if (typeof value !== "string") throw new TypeError("Work item identity must be a string");
  const operation = value === "canvas" ? store.returnToCanvas() : store.activateEditorTab(value);
  void operation.catch((error: unknown) => {
    if (!isApplicationLeaveCancelled(error)) toast.error(getErrorMessage(error));
  });
}
const { t } = useI18n();
const initializationError = shallowRef<Error>();
const initializing = shallowRef(false);
onBeforeUnmount(registerWindowCloseCommit(store.commitActiveSession));
const dock = useContextDock();
const enrichmentJobs = useEnrichmentTaskProjection({
  ready,
  openNode: async (spaceId, nodeId) => {
    await store.openSpace(spaceId);
    if (!store.nodes.some((node) => node.id === nodeId))
      throw new Error(t("spaces.processing.nodeUnavailable"));
    await nextTick();
    selectAndRevealSpaceNode(nodeId);
    dock.openInspect();
  },
});
provide(enrichmentProjectionKey, enrichmentJobs);
const unregisterConversationNavigation = registerConversationRunNavigation(
  async (spaceId, conversationId) => {
    await store.openSpace(spaceId);
    await nextTick();
    dock.openAsk();
    await conversationSession.refreshConversations();
    await conversationSession.selectConversation(conversationId);
  },
);
onBeforeUnmount(unregisterConversationNavigation);

async function initialize() {
  if (initializing.value) return;
  initializing.value = true;
  initializationError.value = undefined;
  try {
    await store.initialize(t("spaces.space.inbox"));
    void restoreConversationRuns().catch((caught: unknown) => {
      toast.error(getErrorMessage(caught));
    });
  } catch (caught) {
    initializationError.value = normalizeError(caught);
  } finally {
    initializing.value = false;
  }
}

onMounted(() => void initialize());
</script>

<template>
  <main class="relative isolate flex h-full min-h-0 flex-col bg-surface-canvas">
    <div v-if="initializationError" class="grid h-full place-items-center p-8 text-center">
      <div class="max-w-sm">
        <AlertTriangle class="mx-auto size-7 text-fg-warning" />
        <h2 class="mt-3 text-title font-semibold">{{ t("spaces.space.load.failed") }}</h2>
        <p class="mt-1 text-body text-fg-secondary">
          {{ t("spaces.space.load.hint") }}
        </p>
        <p class="mt-2 text-caption text-fg-danger">{{ initializationError.message }}</p>
        <UiButton class="mt-4" :disabled="initializing" @click="initialize">{{ t("common.retry") }}</UiButton>
      </div>
    </div>
    <TabsRoot
      v-else-if="store.ready"
      :model-value="activeWorkItem"
      class="flex min-h-0 flex-1 flex-col"
      @update:model-value="activateWorkItem"
    >
      <SpaceWorkTabs />
      <div class="relative min-h-0 flex-1">
        <TabsContent
          value="canvas"
          force-mount
          v-show="surface.kind === 'canvas'"
          class="absolute inset-0 outline-none"
        >
          <SpaceCanvas :is-rendering-enabled="canvasVisible" />
        </TabsContent>
        <SpaceAudioRecordingDock
          class="pointer-events-auto absolute bottom-3 left-1/2 z-50 -translate-x-1/2"
        />
        <section v-if="surface.kind === 'library'" class="absolute inset-0 z-40 bg-surface-canvas">
          <SpaceLibrary />
        </section>
        <SpaceEditorTabs />
        <SpaceRecentlyDeleted
          v-if="store.recentlyDeletedSpaceId !== undefined"
          :key="store.recentlyDeletedSpaceId ?? 'all'"
          :space-id="store.recentlyDeletedSpaceId"
          :open="true"
          @update:open="!$event && (store.recentlyDeletedSpaceId = undefined)"
        />
        <SpaceImageEditLeaveDialog />
      </div>
    </TabsRoot>
  </main>
</template>
