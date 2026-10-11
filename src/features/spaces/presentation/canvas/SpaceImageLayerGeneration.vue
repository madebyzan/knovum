<script setup lang="ts">
import { Layers3, PencilLine, WandSparkles } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, shallowRef, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import { generateImageLayers } from "../../knowledge-derivations/image-layers";
import { useImageLayerAvailability } from "../../knowledge-derivations/use-image-capabilities";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { imageActionAvailability, promptedLayerAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "./node-action-label";
import SpaceImageCustomLayerForm from "./SpaceImageCustomLayerForm.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; compact?: boolean; open: boolean }>();
const emit = defineEmits<{ "update:open": [open: boolean] }>();
const store = useSpaceStore();
const nodeOperations = useNodeOperationGate();
const { t } = useI18n();
const router = useRouter();
const capability = useImageLayerAvailability(toRef(props, "open"));
const prompt = shallowRef("");
const custom = shallowRef(false);
watch(
  () => props.open,
  (open) => {
    if (!open) custom.value = false;
  },
);
const busy = shallowRef(false);
const availability = computed(() =>
  imageActionAvailability({
    sourceReady: Boolean(props.node.data.sourceRevision),
    editing: !nodeOperations.allows("derive"),
    capability: capability.isFetching.value ? undefined : capability.data.value,
    failed: capability.isError.value && !capability.isFetching.value,
    busy: busy.value,
  }),
);
const help = computed(() => imageActionHelp(availability.value, t));
const canSubmit = computed(() => canRunAction(availability.value));
const customAvailability = computed(() => promptedLayerAvailability(availability.value, prompt.value));
const customHelp = computed(() => imageActionHelp(customAvailability.value, t));

async function submit(mode: "automatic" | "custom") {
  if (busy.value || !nodeOperations.allows("derive") || !canSubmit.value) return;
  if (mode === "custom" && !prompt.value.trim()) return;
  setOpen(false);
  busy.value = true;
  try {
    await generateImageLayers(store, props.node, 4, mode === "custom" ? prompt.value : undefined);
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    busy.value = false;
  }
}

function setOpen(value: boolean) {
  emit("update:open", value);
  if (!value) custom.value = false;
}

function openAiSettings() {
  setOpen(false);
  void router.push({ name: "settings", query: { pane: "aiModels", setting: "settings-image-layers" } });
}
</script>

<template>
  <UiPopover
    :open="open"
    side="bottom"
    align="center"
    :class="custom ? 'w-72 p-3 shadow-popover' : 'w-52 p-1.5'"
    @update:open="setOpen"
  >
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :disabled="busy"
          :aria-label="t('spaces.media.imageEditor.layers.title')"
          :title="compact ? t('spaces.media.imageEditor.layers.title') : undefined"
        >
          <Layers3 class="size-4.5" :stroke-width="1.6" />
          <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(Boolean(compact))]">{{
            t("spaces.media.imageEditor.layers.title")
          }}</span>
        </button>
      </ToolbarButton>
    </template>
    <template v-if="!custom">
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-2 text-left text-label text-fg-primary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:cursor-default disabled:opacity-40"
          :disabled="!canSubmit"
          @click="submit('automatic')"
        >
          <WandSparkles class="size-4" :stroke-width="1.6" />
          {{ t("spaces.media.imageEditor.layers.auto") }}
        </button>
        <UiActionHelp
          v-if="help"
          :label="help.label"
          :action-label="help.actionLabel"
          @action="openAiSettings"
        />
      </div>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-label text-fg-primary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
        @click="custom = true"
      >
        <PencilLine class="size-4" :stroke-width="1.6" />
        {{ t("spaces.media.imageEditor.layers.custom") }}
      </button>
    </template>
    <SpaceImageCustomLayerForm
      v-else
      v-model:prompt="prompt"
      :availability="customAvailability"
      :help="customHelp"
      @cancel="setOpen(false)"
      @submit="submit('custom')"
      @open-settings="openAiSettings"
    />
  </UiPopover>
</template>
