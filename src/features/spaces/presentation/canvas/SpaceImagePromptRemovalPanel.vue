<script setup lang="ts">
import { nextTick, onMounted, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { deriveImage } from "../../knowledge-derivations/image-operation";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const space = useSpaceStore();
const panel = useTemplateRef<HTMLElement>("panel");
const description = shallowRef("");
const preparing = shallowRef(false);

function generate() {
  const prompt = description.value.trim();
  if (!prompt || preparing.value) return;
  preparing.value = true;
  void deriveImage(space, props.node, { kind: "remove-object-prompt", prompt }, () => {
    preparing.value = false;
    emit("close");
  }).catch((error) => {
    preparing.value = false;
    toast.error(String(error));
  });
}

onMounted(async () => {
  await nextTick();
  panel.value?.querySelector<HTMLElement>("textarea")?.focus({ preventScroll: true });
});
</script>

<template>
  <section
    ref="panel"
    class="w-72 max-w-[calc(100vw-2rem)] space-y-3 rounded-panel border border-stroke-strong bg-surface-overlay p-3 text-fg-primary shadow-popover"
    :aria-label="t('spaces.media.imageEditor.removeObject.tool')"
  >
    <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.removeObject.tool") }}</h3>
    <label class="grid gap-1 text-label">
      <span>{{ t("spaces.media.imageEditor.removeObject.promptLabel") }}</span>
      <UiTextarea
        v-model="description"
        :placeholder="t('spaces.media.imageEditor.removeObject.promptPlaceholder')"
        :disabled="preparing"
        :maxlength="1000"
        :rows="3"
        class="min-h-20 resize-none"
      />
    </label>
    <p class="text-caption text-fg-secondary">{{ t("spaces.media.imageEditor.removeObject.promptHint") }}</p>
    <footer class="flex gap-2 border-t border-stroke-divider pt-2">
      <UiButton
        variant="secondary"
        size="sm"
        class="min-w-0 flex-1"
        :disabled="preparing"
        @click="emit('close')"
      >
        {{ t("common.cancel") }}
      </UiButton>
      <UiButton
        variant="primary"
        size="sm"
        class="min-w-0 flex-1"
        :disabled="!description.trim() || preparing"
        @click="generate"
      >
        {{ t("spaces.media.imageEditor.removeObject.generate") }}
      </UiButton>
    </footer>
  </section>
</template>
