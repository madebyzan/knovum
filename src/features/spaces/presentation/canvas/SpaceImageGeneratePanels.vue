<script setup lang="ts">
import { Box, Maximize2 } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { rotatedImageSize } from "../../image-editing/canvas-image-renderer";
import { imageEditOf } from "../../image-editing/image-edit";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import { useImageOperationCapabilities } from "../../knowledge-derivations/image-capabilities";
import {
  createImageOperation,
  type ImageOperation,
} from "../../knowledge-derivations/image-operation";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import SpaceImageAnglePanel from "./SpaceImageAnglePanel.vue";
import { imageActionAvailability, upscaleAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload>; mode: "upscale" | "angle" }>();
const { t } = useI18n();
const router = useRouter();
const store = useSpaceStore();
const imageEditSession = useImageEditSession();
const dimensions = computed(() => {
  const loaded = imageEditSession.dimensionsFor(props.node.id);
  const width = props.node.data.intrinsicWidth ?? loaded?.width;
  const height = props.node.data.intrinsicHeight ?? loaded?.height;
  if (!width || !height) return undefined;
  const edit = imageEditOf(props.node.data);
  return rotatedImageSize(width * edit.crop.width, height * edit.crop.height, edit.rotation);
});
const capabilities = useImageOperationCapabilities();
const busy = shallowRef(false);
const upscaleOpen = shallowRef(false);
const angleOpen = shallowRef(false);
const sizes = [2048, 4096, 8192] as const;
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-100 hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring disabled:cursor-not-allowed disabled:opacity-45";
const baseAvailability = computed(() => imageActionAvailability(
  Boolean(props.node.data.sourceRevision),
  imageEditSession.active?.nodeId === props.node.id,
  capabilities.isFetching.value
    ? undefined
    : props.mode === "upscale" ? capabilities.data.value?.upscale : capabilities.data.value?.multiAngle,
  capabilities.isError.value && !capabilities.isFetching.value,
  busy.value,
));
const help = computed(() => imageActionHelp(baseAvailability.value, t));

function targetDimensions(longEdge: number) {
  const source = dimensions.value;
  if (!source) return undefined;
  const scale = longEdge / Math.max(source.width, source.height);
  return `${Math.round(source.width * scale)} × ${Math.round(source.height * scale)}`;
}

function optionAvailability(size: number) {
  return upscaleAvailability(baseAvailability.value, dimensions.value, size);
}

function openAiSettings() {
  angleOpen.value = false;
  upscaleOpen.value = false;
  void router.push({ name: "settings", query: { pane: "aiModels" } });
}

async function run(operation: ImageOperation) {
  if (!canRunAction(baseAvailability.value)) return;
  if (operation.kind === "upscale" && !canRunAction(optionAvailability(operation.targetLongEdge))) return;
  upscaleOpen.value = false;
  angleOpen.value = false;
  busy.value = true;
  try {
    await createImageOperation(store, props.node, operation);
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <UiPopover v-if="mode === 'upscale'" v-model:open="upscaleOpen" side="bottom" align="start" class="w-48 p-1" @update:open="(value: boolean) => { if (value) void capabilities.refetch(); }">
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          :class="commandClass"
          :disabled="busy"
        >
          <Maximize2 class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.media.upscale") }}
        </button>
      </ToolbarButton>
    </template>
    <UiTooltip v-for="size in sizes" :key="size" :text="imageActionHelp(optionAvailability(size), t)?.label ?? ''" :disabled="canRunAction(optionAvailability(size))" side="right">
      <button type="button" class="flex h-(--knovum-control-height-lg) w-full items-center justify-between gap-2 rounded-control px-2 text-left text-label outline-none hover:bg-control-hover focus-visible:ring-2 focus-visible:ring-focus-ring aria-disabled:cursor-not-allowed aria-disabled:opacity-45" :aria-disabled="!canRunAction(optionAvailability(size))" @click="run({ kind: 'upscale', targetLongEdge: size })">
        <span class="shrink-0 font-medium text-foreground">{{ size / 1024 }}K</span>
        <span v-if="targetDimensions(size)" class="tabular-nums text-foreground-tertiary">{{ targetDimensions(size) }}</span>
      </button>
    </UiTooltip>
    <div v-if="help?.actionLabel" class="flex justify-end border-t border-divider pt-1">
      <button type="button" class="px-2 py-1 text-caption text-accent hover:underline" @click="openAiSettings">{{ help.actionLabel }}</button>
    </div>
  </UiPopover>
  <UiPopover v-if="mode === 'angle'" v-model:open="angleOpen" side="bottom" align="center" class="p-3" @update:open="(value: boolean) => { if (value) void capabilities.refetch(); }">
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          :class="commandClass"
          :disabled="busy"
        >
          <Box class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.media.multiAngle") }}
        </button>
      </ToolbarButton>
    </template>
    <SpaceImageAnglePanel
      :path="node.data.assetPath"
      :revision="node.data.sourceRevision"
      :edit="imageEditOf(node.data)"
      :busy="busy"
      :availability="baseAvailability"
      :help="help"
      @cancel="angleOpen = false"
      @open-settings="openAiSettings"
      @apply="run"
    />
  </UiPopover>
</template>
