<script setup lang="ts">
import { Layers3, PencilLine, WandSparkles } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import { useImageLayerCapability } from "../../knowledge-derivations/image-capabilities";
import { createImageLayers } from "../../knowledge-derivations/image-layers";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { imageActionAvailability, promptedLayerAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";
import SpaceImageCustomLayerForm from "./SpaceImageCustomLayerForm.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; compact?: boolean; open: boolean }>();
const emit = defineEmits<{ "update:open": [open: boolean] }>();
const store = useSpaceStore();
const imageEditSession = useImageEditSession();
const { t } = useI18n();
const router = useRouter();
const capability = useImageLayerCapability();
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
  imageActionAvailability(
    Boolean(props.node.data.sourceRevision),
    imageEditSession.active?.nodeId === props.node.id,
    capability.isFetching.value ? undefined : capability.data.value,
    capability.isError.value && !capability.isFetching.value,
    busy.value,
  ),
);
const help = computed(() => imageActionHelp(availability.value, t));
const canSubmit = computed(() => canRunAction(availability.value));
const customAvailability = computed(() => promptedLayerAvailability(availability.value, prompt.value));
const customHelp = computed(() => imageActionHelp(customAvailability.value, t));
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-100 hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring disabled:cursor-not-allowed disabled:opacity-45";

async function submit(mode: "automatic" | "custom") {
  if (busy.value || !canSubmit.value) return;
  if (mode === "custom" && !prompt.value.trim()) return;
  setOpen(false);
  busy.value = true;
  try {
    await createImageLayers(store, props.node, 4, mode === "custom" ? prompt.value : undefined);
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}

function setOpen(value: boolean) {
  emit("update:open", value);
  if (!value) custom.value = false;
  if (value) void capability.refetch();
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
          :class="commandClass"
          :disabled="busy"
          :aria-label="t('spaces.media.layerSplit')"
          :title="compact ? t('spaces.media.layerSplit') : undefined"
        >
          <Layers3 class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          <span :class="[nodeActionLabelClass, nodeActionLabelState(Boolean(compact))]">{{
            t("spaces.media.layerSplit")
          }}</span>
        </button>
      </ToolbarButton>
    </template>
    <template v-if="!custom">
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="flex min-w-0 flex-1 items-center gap-2 rounded-control px-2 py-2 text-left text-label text-foreground hover:bg-control-hover focus-visible:ring-2 focus-visible:ring-focus-ring disabled:cursor-not-allowed disabled:opacity-40"
          :disabled="!canSubmit"
          @click="submit('automatic')"
        >
          <WandSparkles class="size-(--knovum-icon-size-md)" :stroke-width="1.6" />
          {{ t("spaces.media.autoSplit") }}
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
        class="flex w-full items-center gap-2 rounded-control px-2 py-2 text-left text-label text-foreground hover:bg-control-hover focus-visible:ring-2 focus-visible:ring-focus-ring"
        @click="custom = true"
      >
        <PencilLine class="size-(--knovum-icon-size-md)" :stroke-width="1.6" />
        {{ t("spaces.media.customSplit") }}
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
