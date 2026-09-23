<script setup lang="ts">
import { Layers3, PencilLine, ScanLine, WandSparkles } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import { useImageLayerCapability } from "../../knowledge-derivations/image-capabilities";
import { createImageLayers } from "../../knowledge-derivations/image-layers";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { imageActionAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const store = useSpaceStore();
const imageEditSession = useImageEditSession();
const { t } = useI18n();
const router = useRouter();
const capability = useImageLayerCapability();
const open = shallowRef(false);
const prompt = shallowRef("");
const custom = shallowRef(false);
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
const customAvailability = computed(() =>
  canSubmit.value && !prompt.value.trim()
    ? { state: "blocked" as const, reason: "prompt_required" as const }
    : availability.value,
);
const customHelp = computed(() => imageActionHelp(customAvailability.value, t));
const showCustomHelp = computed(() => {
  const state = customAvailability.value;
  return state.state === "blocked" && state.reason !== "prompt_required" && Boolean(customHelp.value);
});
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-100 hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring disabled:cursor-not-allowed disabled:opacity-45";

async function submit(mode: "automatic" | "custom") {
  if (busy.value || !canSubmit.value) return;
  if (mode === "custom" && !prompt.value.trim()) return;
  open.value = false;
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
  open.value = value;
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
        <button type="button" :class="commandClass" :disabled="busy">
          <Layers3 class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.media.layerSplit") }}
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
    <section v-else class="text-foreground" :aria-label="t('spaces.media.customSplit')">
      <header class="flex items-center justify-between gap-2 px-1 pb-2">
        <h3 class="text-body font-semibold">{{ t("spaces.media.splitLayersPanel") }}</h3>
        <UiTooltip :text="t('spaces.media.selectLayerContentUnavailable')" side="top">
          <button
            type="button"
            class="inline-flex cursor-not-allowed items-center gap-1 text-label text-foreground-tertiary"
            :aria-label="t('spaces.media.selectLayerContent')"
            aria-disabled="true"
          >
            <ScanLine class="size-(--knovum-icon-size-sm)" :stroke-width="1.5" />
            {{ t("spaces.media.selectLayerContent") }}
          </button>
        </UiTooltip>
      </header>
      <textarea
        v-model="prompt"
        rows="2"
        maxlength="1000"
        :aria-label="t('spaces.media.layerPrompt')"
        class="block min-h-18 w-full resize-none rounded-control bg-transparent px-1 py-2 text-label text-foreground outline-none placeholder:text-foreground-tertiary focus-visible:ring-2 focus-visible:ring-focus-ring"
        :placeholder="t('spaces.media.layerPromptPlaceholder')"
      />
      <footer class="mt-1 flex items-center gap-2">
        <UiButton
          variant="secondary"
          size="md"
          class="min-w-0 flex-1 border border-border bg-raised"
          @click="setOpen(false)"
          >{{ t("common.cancel") }}</UiButton
        >
        <UiButton
          variant="primary"
          size="md"
          class="min-w-0 flex-1 bg-foreground text-canvas hover:bg-foreground hover:opacity-90"
          :disabled="!canRunAction(customAvailability)"
          @click="submit('custom')"
          >{{ t("spaces.media.layerSplitAction") }}</UiButton
        >
        <UiActionHelp
          v-if="showCustomHelp && customHelp"
          :label="customHelp.label"
          :action-label="customHelp.actionLabel"
          @action="openAiSettings"
        />
      </footer>
    </section>
  </UiPopover>
</template>
