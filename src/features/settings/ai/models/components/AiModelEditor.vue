<script setup lang="ts">
import { cloneDeep } from "es-toolkit";
import { useId } from "reka-ui";
import { computed, reactive, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { aiModelDefinitionSchema } from "../../../../../foundation/ai/schema/models";
import type { AiModelDefinition } from "../../../../../shared/contracts/generated/AiModelDefinition";
import type { ExecutionCapability } from "../../../../../shared/contracts/generated/ExecutionCapability";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import UiSwitch from "../../../../../shared/ui/UiSwitch.vue";
import AiModelDefinitionEditor from "./AiModelDefinitionEditor.vue";

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
const builtinAgentSwitchId = useId();
const draft = reactive<AiModelDefinition>(
  props.model
    ? cloneDeep(props.model)
    : { id: crypto.randomUUID(), name: "", model: "", capabilities: [], builtinAgent: false },
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
    builtinAgent: draft.builtinAgent,
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
      <section
        v-if="builtinAgentSupported && draft.capabilities.includes('text-generation')"
        class="mt-5 grid gap-3 border-t border-stroke-divider pt-4"
      >
        <div class="flex items-center justify-between gap-3">
          <label :for="builtinAgentSwitchId" class="text-label text-fg-primary">{{
            t("settings.ai.models.builtinAgent.enabled")
          }}</label>
          <UiSwitch
            :id="builtinAgentSwitchId"
            v-model="draft.builtinAgent"
            :disabled="busy"
            :label="t('settings.ai.models.builtinAgent.enabled')"
          />
        </div>
        <p class="text-caption text-fg-secondary">{{ t("settings.ai.models.builtinAgent.help") }}</p>
      </section>
    </fieldset>
    <p v-if="error" class="mt-3 text-caption text-fg-danger" role="alert">{{ error }}</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="busy" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="busy" @click="save">{{ t("common.save") }}</UiButton>
    </template>
  </UiDialog>
</template>
