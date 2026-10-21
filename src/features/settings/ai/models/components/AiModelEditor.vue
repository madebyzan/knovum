<script setup lang="ts">
import { cloneDeep } from "es-toolkit";
import { computed, reactive, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { aiModelDefinitionSchema } from "../../../../../foundation/ai/schema/models";
import type { AiModelDefinition } from "../../../../../shared/contracts/generated/AiModelDefinition";
import type { BuiltinAgentModelSettings } from "../../../../../shared/contracts/generated/BuiltinAgentModelSettings";
import type { ExecutionCapability } from "../../../../../shared/contracts/generated/ExecutionCapability";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import AiModelDefinitionEditor from "./AiModelDefinitionEditor.vue";
import BuiltinAgentModelSettingsEditor from "./BuiltinAgentModelSettingsEditor.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  connectionName: string;
  builtinAgentSupported: boolean;
  model?: AiModelDefinition | undefined;
  allowedCapabilities: readonly ExecutionCapability[];
  protected: boolean;
  busy: boolean;
  error?: string | undefined;
}>();
const emit = defineEmits<{ save: [model: AiModelDefinition]; dirty: [value: boolean] }>();
const { t } = useI18n();
const draft = reactive<AiModelDefinition & { builtinAgent: BuiltinAgentModelSettings | null }>(
  props.model
    ? cloneDeep({ ...props.model, builtinAgent: props.model.builtinAgent ?? null })
    : { id: crypto.randomUUID(), name: "", model: "", capabilities: [], builtinAgent: null },
);
const initial = JSON.stringify(draft);
const changed = computed(() => JSON.stringify(draft) !== initial);
const attempted = shallowRef(false);
const valid = computed(
  () =>
    aiModelDefinitionSchema.safeParse(draft).success &&
    draft.capabilities.every((capability) => props.allowedCapabilities.includes(capability)),
);
watch(changed, (value) => emit("dirty", value), { immediate: true });

function save() {
  if (props.model && !changed.value) {
    open.value = false;
    return;
  }
  attempted.value = true;
  if (props.busy || !valid.value) return;
  emit("save", {
    id: draft.id,
    name: draft.name.trim(),
    model: draft.model.trim(),
    capabilities: [...draft.capabilities],
    builtinAgent: draft.builtinAgent ? { ...draft.builtinAgent } : null,
  });
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t(props.model ? 'settings.ai.models.edit' : 'settings.ai.models.add')"
    :description="connectionName"
  >
    <fieldset :disabled="busy" class="border-0 p-0">
      <AiModelDefinitionEditor
        :model-value="draft"
        :protected="protected"
        :allowed-capabilities="allowedCapabilities"
        :show-errors="attempted"
        @update:model-value="Object.assign(draft, $event)"
      />
      <BuiltinAgentModelSettingsEditor
        v-if="builtinAgentSupported && draft.capabilities.includes('text-generation')"
        v-model="draft.builtinAgent"
        :disabled="busy"
        :show-errors="attempted"
      />
    </fieldset>
    <p v-if="error" class="mt-3 text-caption text-fg-danger" role="alert">{{ error }}</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="busy" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="busy" @click="save">{{ t("common.save") }}</UiButton>
    </template>
  </UiDialog>
</template>
