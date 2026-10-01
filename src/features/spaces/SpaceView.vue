<script setup lang="ts">
import { AlertTriangle } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { nextTick, onBeforeUnmount, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { normalizeError } from "../../foundation/errors/app-error";
import { errorMessage } from "../../foundation/errors/error-format";
import UiButton from "../../shared/ui/UiButton.vue";
import { useContextDock } from "./canvas/use-context-dock";
import {
  registerConversationRunNavigation,
  restoreConversationRuns,
} from "./conversation/conversation-run-observation";
import { useConversationSessionStore } from "./conversation/conversation-session-store";
import { useDeferredEditorMount } from "./editor/use-deferred-editor-mount";
import SpaceCanvas from "./presentation/canvas/SpaceCanvas.vue";
import SpaceRecentlyDeleted from "./presentation/deletion/SpaceRecentlyDeleted.vue";
import SpaceTiptapEditor from "./presentation/editor/SpaceTiptapEditor.vue";
import SpaceLibrary from "./presentation/library/SpaceLibrary.vue";
import SpaceAudioRecordingDock from "./presentation/media/SpaceAudioRecordingDock.vue";
import SpaceEditorLoading from "./presentation/SpaceEditorLoading.vue";
import SpaceVideoEditor from "./presentation/video-editor/SpaceVideoEditor.vue";
import { restoreResourceRuns } from "./runs/resource-run-observation";
import { useSpaceStore } from "./session/space-store";
import { useSpaceSessionLifecycle } from "./workspace/use-space-session-lifecycle";

const store = useSpaceStore();
const { focusedEditorId } = storeToRefs(store);
const { t } = useI18n();
const initializationError = shallowRef<Error>();
const initializing = shallowRef(false);
const mountedEditorId = useDeferredEditorMount(focusedEditorId);
useSpaceSessionLifecycle(store.commitActiveSession);
const dock = useContextDock();
const unregisterConversationNavigation = registerConversationRunNavigation(
  async (spaceId, conversationId) => {
    await store.openSpace(spaceId);
    await nextTick();
    dock.openAsk();
    const assistant = useConversationSessionStore();
    await assistant.refreshConversations();
    await assistant.selectConversation(conversationId);
  },
);
onBeforeUnmount(unregisterConversationNavigation);

async function initialize() {
  if (initializing.value) return;
  initializing.value = true;
  initializationError.value = undefined;
  try {
    await store.initialize(t("spaces.space.inbox"));
    void restoreResourceRuns(t("spaces.creation.shared.resourceTask")).catch((caught: unknown) => {
      toast.error(errorMessage(caught));
    });
    void restoreConversationRuns().catch((caught: unknown) => {
      toast.error(errorMessage(caught));
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
  <main class="relative isolate h-full min-h-0 bg-surface-canvas">
    <div v-if="initializationError" class="grid h-full place-items-center p-8 text-center">
      <div class="max-w-sm">
        <AlertTriangle class="mx-auto size-7 text-fg-warning" />
        <h2 class="mt-3 text-title font-semibold">{{ t("spaces.space.loadFailed") }}</h2>
        <p class="mt-1 text-body text-fg-secondary">
          {{ t("spaces.space.loadFailedHint") }}
        </p>
        <p class="mt-2 text-caption text-fg-danger">{{ initializationError.message }}</p>
        <UiButton class="mt-4" :disabled="initializing" @click="initialize">{{ t("common.retry") }}</UiButton>
      </div>
    </div>
    <template v-else-if="store.ready">
      <SpaceCanvas />
      <SpaceAudioRecordingDock class="pointer-events-auto absolute bottom-3 left-1/2 z-50 -translate-x-1/2" />
      <section v-if="store.workspaceMode === 'library'" class="absolute inset-0 z-40 bg-surface-canvas">
        <SpaceLibrary />
      </section>
      <SpaceEditorLoading v-if="focusedEditorId && !mountedEditorId" />
      <SpaceTiptapEditor v-if="mountedEditorId" :key="mountedEditorId" />
      <SpaceRecentlyDeleted
        v-if="store.recentlyDeletedSpaceId !== undefined"
        :key="store.recentlyDeletedSpaceId ?? 'all'"
        :space-id="store.recentlyDeletedSpaceId"
        :open="true"
        @update:open="!$event && (store.recentlyDeletedSpaceId = undefined)"
      />
      <SpaceVideoEditor v-if="store.videoEditorNodeId" />
    </template>
  </main>
</template>
