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

const props = defineProps<{ availability: ImageActionAvailability; help?: ImageActionHelp | undefined }>();
const prompt = defineModel<string>("prompt", { required: true });
const emit = defineEmits<{ cancel: []; submit: []; openSettings: [] }>();
const { t } = useI18n();
const showHelp = computed(
  () => props.availability.state === "blocked" && props.availability.reason !== "prompt_required",
);
</script>

<template>
  <section class="text-fg-primary" :aria-label="t('spaces.media.imageEditor.layers.custom')">
    <header class="flex items-center justify-between gap-2 px-1 pb-2">
      <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.layers.panel") }}</h3>
      <UiTooltip :text="t('spaces.media.imageEditor.layers.selectContentUnavailable')" side="top">
        <button
          type="button"
          class="inline-flex cursor-not-allowed items-center gap-1 text-label text-fg-tertiary"
          :aria-label="t('spaces.media.imageEditor.layers.selectContent')"
          aria-disabled="true"
        >
          <ScanLine class="size-3.5" :stroke-width="1.5" />
          {{ t("spaces.media.imageEditor.layers.selectContent") }}
        </button>
      </UiTooltip>
    </header>
    <textarea
      v-model="prompt"
      rows="2"
      maxlength="1000"
      :aria-label="t('spaces.media.imageEditor.layers.prompt')"
      class="block min-h-18 w-full resize-none rounded-md bg-transparent px-1 py-2 text-label text-fg-primary outline-none placeholder:text-fg-tertiary focus-visible:ring-2 focus-visible:ring-stroke-focus"
      :placeholder="t('spaces.media.imageEditor.layers.promptPlaceholder')"
    />
    <footer class="mt-1 flex items-center gap-2">
      <UiButton
        variant="secondary"
        size="md"
        class="min-w-0 flex-1 border border-stroke-default bg-surface-raised"
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton
        variant="primary"
        size="md"
        class="min-w-0 flex-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="!canRunAction(availability)"
        @click="emit('submit')"
        >{{ t("spaces.media.imageEditor.layers.split") }}</UiButton
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
