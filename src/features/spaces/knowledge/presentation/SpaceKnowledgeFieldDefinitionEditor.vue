<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpaceObjectFieldDefinition, SpaceObjectFieldKind } from "../model/object-types";

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
    label: t(`spaces.knowledge.fields.kindLabels.${kind}`),
  })),
);
</script>

<template>
  <div class="col-span-full grid grid-cols-subgrid items-center gap-y-2">
    <div class="contents">
      <UiInput
        :model-value="field.label"
        maxlength="128"
        :aria-label="t('spaces.knowledge.fields.label')"
        @update:model-value="emit('label', $event)"
      />
      <UiInput
        :model-value="field.key"
        maxlength="128"
        :aria-label="t('spaces.knowledge.fields.key')"
        class="font-mono"
        @update:model-value="emit('key', $event)"
      />
      <UiSelect
        :disabled="disabled"
        :model-value="field.kind"
        :label="t('spaces.knowledge.fields.kind')"
        :options="kindOptions"
        @update:model-value="emit('kind', $event)"
      />
      <label class="flex h-(--knovum-control-height-md) items-center justify-center">
        <UiCheckbox
          :disabled="disabled"
          :model-value="Boolean(field.required)"
          :label="t('spaces.knowledge.fields.required')"
          @update:model-value="emit('required', $event)"
        />
      </label>
      <UiIconButton :disabled="disabled" :label="t('spaces.knowledge.fields.remove')" @click="emit('remove')">
        <Trash2 class="size-3.5" />
      </UiIconButton>
    </div>
    <label v-if="field.kind === 'enum'" class="col-span-3 grid gap-1 pb-1 text-caption text-fg-secondary">
      <span>{{ t("spaces.knowledge.fields.options") }}</span>
      <UiInput
        :model-value="field.options?.join(', ') ?? ''"
        :placeholder="t('spaces.knowledge.fields.options')"
        @update:model-value="emit('options', $event)"
      />
    </label>
  </div>
</template>
