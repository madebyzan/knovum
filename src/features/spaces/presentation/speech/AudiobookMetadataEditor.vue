<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { AudiobookMetadata } from "../../contracts/generated/AudiobookMetadata";
import { useSpaceStore } from "../../session/space-store";

defineProps<{ disabled: boolean }>();
const model = defineModel<AudiobookMetadata>({ required: true });
const { t } = useI18n();
const store = useSpaceStore();
const covers = computed(() =>
  store.nodes.flatMap(({ data }) =>
    data.kind === "image" && data.assetPath && data.sourceRevision
      ? [{ value: data.assetPath, label: data.title, sourceRevision: data.sourceRevision }]
      : [],
  ),
);
const options = computed(() => [
  { value: "none", label: t("spaces.speech.audiobook.noCover") },
  ...covers.value,
  ...(model.value.cover && !covers.value.some((cover) => cover.value === model.value.cover?.assetPath)
    ? [{ value: model.value.cover.assetPath, label: t("spaces.speech.audiobook.boundCover") }]
    : []),
]);
const cover = computed({
  get: () => model.value.cover?.assetPath ?? "none",
  set: (value: string) => {
    const selected = covers.value.find((item) => item.value === value);
    model.value = {
      ...model.value,
      cover: selected ? { assetPath: selected.value, sourceRevision: selected.sourceRevision } : null,
    };
  },
});
function updateText(field: "title" | "author" | "narrator", value: string) {
  model.value = { ...model.value, [field]: value };
}
</script>

<template>
  <section class="grid grid-cols-2 gap-x-2 gap-y-3">
    <label class="col-span-2 block min-w-0 space-y-1 text-caption text-fg-secondary"
      >{{ t("spaces.speech.audiobook.title")
      }}<UiInput
        :model-value="model.title"
        size="sm"
        :disabled
        @update:model-value="updateText('title', $event)"
    /></label>
    <label class="block min-w-0 space-y-1 text-caption text-fg-secondary"
      >{{ t("spaces.speech.audiobook.author")
      }}<UiInput
        :model-value="model.author"
        size="sm"
        :disabled
        @update:model-value="updateText('author', $event)"
    /></label>
    <label class="block min-w-0 space-y-1 text-caption text-fg-secondary"
      >{{ t("spaces.speech.audiobook.narrator")
      }}<UiInput
        :model-value="model.narrator"
        size="sm"
        :disabled
        @update:model-value="updateText('narrator', $event)"
    /></label>
    <label class="col-span-2 block min-w-0 space-y-1 text-caption text-fg-secondary">
      <span>{{ t("spaces.speech.audiobook.cover") }}</span>
      <UiSelect v-model="cover" size="sm" :label="t('spaces.speech.audiobook.cover')" :options :disabled />
    </label>
  </section>
</template>
