<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpaceObjectFieldDefinition, SpaceObjectFieldKind } from "../../knowledge/model/object-types";
import SpaceKnowledgeFieldDefinitionEditor from "./SpaceKnowledgeFieldDefinitionEditor.vue";

defineProps<{
  fields: readonly SpaceObjectFieldDefinition[];
  fieldKinds: readonly SpaceObjectFieldKind[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  add: [];
  kind: [field: SpaceObjectFieldDefinition, kind: SpaceObjectFieldKind];
  options: [field: SpaceObjectFieldDefinition, value: string];
  required: [field: SpaceObjectFieldDefinition, value: boolean];
  label: [field: SpaceObjectFieldDefinition, value: string];
  key: [field: SpaceObjectFieldDefinition, value: string];
  remove: [index: number];
}>();
const { t } = useI18n();
const identityByField = new WeakMap<SpaceObjectFieldDefinition, string>();

function identity(field: SpaceObjectFieldDefinition) {
  let id = identityByField.get(field);
  if (!id) {
    id = crypto.randomUUID();
    identityByField.set(field, id);
  }
  return id;
}
</script>

<template>
  <div class="mt-4">
    <h3 class="mb-2 text-label font-medium">{{ t("spaces.knowledge.fields.title") }}</h3>
    <p v-if="!fields.length" class="py-3 text-caption text-fg-secondary">
      {{ t("spaces.knowledge.types.emptyFields") }}
    </p>
    <div class="overflow-x-auto">
      <div
        class="grid min-w-[30rem] grid-cols-[minmax(0,1fr)_minmax(0,0.8fr)_8rem_3rem_var(--knovum-control-height-md)] items-center gap-2"
      >
        <template v-if="fields.length">
          <span class="text-caption text-fg-secondary">{{ t("spaces.knowledge.fields.label") }}</span>
          <span class="text-caption text-fg-secondary">{{ t("spaces.knowledge.fields.key") }}</span>
          <span class="text-caption text-fg-secondary">{{ t("spaces.knowledge.fields.kind") }}</span>
          <span class="text-center text-caption text-fg-secondary">{{
            t("spaces.knowledge.fields.required")
          }}</span>
          <span aria-hidden="true"></span>
        </template>
        <SpaceKnowledgeFieldDefinitionEditor
          v-for="(field, index) in fields"
          :key="identity(field)"
          :field="field"
          :disabled="disabled"
          :field-kinds="fieldKinds"
          @kind="emit('kind', field, $event)"
          @options="emit('options', field, $event)"
          @required="emit('required', field, $event)"
          @label="emit('label', field, $event)"
          @key="emit('key', field, $event)"
          @remove="emit('remove', index)"
        />
      </div>
    </div>
    <UiButton class="mt-2" :disabled="fields.length >= 64" size="sm" variant="ghost" @click="emit('add')">
      <Plus class="size-3" />{{ t("spaces.knowledge.fields.add") }}
    </UiButton>
  </div>
</template>
