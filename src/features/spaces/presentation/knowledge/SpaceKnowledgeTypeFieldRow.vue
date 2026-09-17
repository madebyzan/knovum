<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpaceObjectFieldDefinition, SpaceObjectFieldKind } from "../../knowledge/model/object-types";

const props = defineProps<{
  field: SpaceObjectFieldDefinition;
  fieldKinds: readonly SpaceObjectFieldKind[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  remove: [];
  kind: [kind: SpaceObjectFieldKind];
  options: [value: string];
  required: [value: boolean];
  label: [value: string];
  key: [value: string];
}>();
const { t } = useI18n();
const kindOptions = computed(() =>
  props.fieldKinds.map((kind) => ({
    value: kind,
    label: t(`spaces.knowledge.knowledgeFieldKindLabel.${kind}`),
  })),
);
</script>

<template>
  <div class="col-span-full grid grid-cols-subgrid items-center gap-y-2">
    <div class="contents">
      <UiInput
        :model-value="field.label"
        maxlength="128"
        :aria-label="t('spaces.knowledge.knowledgeFieldLabel')"
        @update:model-value="emit('label', $event)"
      />
      <UiInput
        :model-value="field.key"
        maxlength="128"
        :aria-label="t('spaces.knowledge.knowledgeFieldKey')"
        class="font-mono"
        @update:model-value="emit('key', $event)"
      />
      <UiSelect
        :disabled="disabled"
        :model-value="field.kind"
        :label="t('spaces.knowledge.knowledgeFieldKind')"
        :options="kindOptions"
        @update:model-value="emit('kind', $event as SpaceObjectFieldKind)"
      />
      <label class="flex h-(--knovum-control-height-md) items-center justify-center">
        <UiCheckbox
          :disabled="disabled"
          :model-value="Boolean(field.required)"
          :label="t('spaces.knowledge.knowledgeFieldRequired')"
          @update:model-value="emit('required', $event)"
        />
      </label>
      <UiIconButton
        :disabled="disabled"
        :label="t('spaces.knowledge.removeKnowledgeField')"
        @click="emit('remove')"
      >
        <Trash2 class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
    </div>
    <label
      v-if="field.kind === 'enum'"
      class="col-span-3 grid gap-1 pb-1 text-caption text-foreground-secondary"
    >
      <span>{{ t("spaces.knowledge.knowledgeFieldOptions") }}</span>
      <UiInput
        :model-value="field.options?.join(', ') ?? ''"
        :placeholder="t('spaces.knowledge.knowledgeFieldOptions')"
        @update:model-value="emit('options', $event)"
      />
    </label>
  </div>
</template>
