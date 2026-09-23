<script setup lang="ts">
import { ScanLine } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type { ImageActionAvailability } from "./image-action-availability";
import type { ImageActionHelp } from "./image-action-help";

const props = defineProps<{ availability: ImageActionAvailability; help?: ImageActionHelp }>();
const prompt = defineModel<string>("prompt", { required: true });
const emit = defineEmits<{ cancel: []; submit: []; openSettings: [] }>();
const { t } = useI18n();
const showHelp = computed(
  () => props.availability.state === "blocked" && props.availability.reason !== "prompt_required",
);
</script>

<template>
  <section class="text-foreground" :aria-label="t('spaces.media.customSplit')">
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
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton
        variant="primary"
        size="md"
        class="min-w-0 flex-1 bg-foreground text-canvas hover:bg-foreground hover:opacity-90"
        :disabled="!canRunAction(availability)"
        @click="emit('submit')"
        >{{ t("spaces.media.layerSplitAction") }}</UiButton
      >
      <UiActionHelp
        v-if="showHelp && help"
        :label="help.label"
        :action-label="help.actionLabel"
        @action="emit('openSettings')"
      />
    </footer>
  </section>
</template>
