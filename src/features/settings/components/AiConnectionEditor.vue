<script setup lang="ts">
import { computed, reactive, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { requireProvider } from "../../../foundation/ai/providers";
import { aiConnectionBaseUrlSchema, aiConnectionSettingsSchema } from "../../../foundation/ai/schema/models";
import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import type { ProviderDescriptor } from "../../../shared/contracts/generated/ProviderDescriptor";
import type { ProviderKind } from "../../../shared/contracts/generated/ProviderKind";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import {
  createAiConnectionDraft,
  finalizeAiConnectionDraft,
  type AiConnectionSecretChange,
  type AiConnectionSubmission,
} from "../ai-connection-draft";
import { providerOptions } from "../ai-provider-options";
import AiConnectionCredentialField from "./AiConnectionCredentialField.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  connection?: AiConnectionSettings | undefined;
  providers: readonly ProviderDescriptor[];
  busy: boolean;
  error?: string | undefined;
}>();
const emit = defineEmits<{ save: [submission: AiConnectionSubmission]; dirty: [value: boolean] }>();
const { t } = useI18n();
const firstProvider = props.connection
  ? requireProvider(props.providers, props.connection.provider)
  : props.providers[0]!;
const draft = reactive<AiConnectionSettings>(
  props.connection
    ? createAiConnectionDraft(props.connection)
    : {
        id: crypto.randomUUID(),
        name: firstProvider.displayName,
        provider: firstProvider.id,
        baseUrl: firstProvider.defaultBaseUrl,
        models: [],
      },
);
const attempted = shallowRef(false);
const secret = shallowRef<AiConnectionSecretChange>();
const provider = computed(() => requireProvider(props.providers, draft.provider));
const options = computed(() => providerOptions(props.providers));
const validUrl = computed(() => aiConnectionBaseUrlSchema.safeParse(draft.baseUrl).success);
const valid = computed(() => aiConnectionSettingsSchema.safeParse(draft).success && validUrl.value);
const initialName = draft.name.trim();
const initialBaseUrl = draft.baseUrl.trim();
const initialProvider = draft.provider;
const changed = computed(
  () =>
    secret.value === null ||
    Boolean(secret.value?.trim()) ||
    draft.name.trim() !== initialName ||
    draft.baseUrl.trim() !== initialBaseUrl ||
    draft.provider !== initialProvider,
);
watch(changed, (value) => emit("dirty", value), { immediate: true });

function selectProvider(id: ProviderKind) {
  const previous = provider.value;
  const next = requireProvider(props.providers, id);
  if (draft.name === previous.displayName) draft.name = next.displayName;
  draft.provider = id;
  draft.baseUrl = next.defaultBaseUrl;
}

function save() {
  if (props.connection && !changed.value) {
    open.value = false;
    return;
  }
  attempted.value = true;
  if (!valid.value || props.busy) return;
  emit("save", {
    connection: finalizeAiConnectionDraft(draft),
    secret: typeof secret.value === "string" ? secret.value.trim() || undefined : secret.value,
  });
}
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="t(props.connection ? 'settings.ai.editConnection' : 'settings.ai.addConnection')"
  >
    <fieldset :disabled="busy" class="min-w-0 border-0 p-0">
      <div class="grid gap-4">
        <div v-if="!props.connection" class="grid gap-1.5 text-caption font-medium text-fg-secondary">
          {{ t("settings.ai.provider") }}
          <UiSelect
            :model-value="draft.provider"
            :label="t('settings.ai.provider')"
            :options="options"
            @update:model-value="selectProvider($event)"
          />
        </div>
        <label class="grid gap-1.5 text-caption font-medium text-fg-secondary">
          {{ t("settings.ai.connectionName") }}
          <UiInput v-model="draft.name" />
          <span v-if="attempted && !draft.name.trim()" class="text-fg-danger">{{
            t("settings.ai.requiredField")
          }}</span>
        </label>
        <label class="grid min-w-0 gap-1.5 text-caption font-medium text-fg-secondary">
          {{ t("settings.ai.baseUrl") }}
          <UiInput v-model="draft.baseUrl" />
          <span v-if="attempted && !validUrl" class="text-fg-danger">{{
            t("settings.ai.invalidBaseUrl")
          }}</span>
        </label>
        <AiConnectionCredentialField
          v-model="secret"
          :connection-id="draft.id"
          :existing="Boolean(props.connection)"
          :disabled="busy"
        />
      </div>
    </fieldset>
    <p v-if="props.error" class="mt-3 text-caption text-fg-danger" role="alert">{{ props.error }}</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="busy" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="busy" @click="save">{{
        busy
          ? t("settings.ai.savingConnection")
          : props.connection && !changed
            ? t("settings.ai.finishEditing")
            : t("settings.ai.saveConnection")
      }}</UiButton>
    </template>
  </UiDialog>
</template>
