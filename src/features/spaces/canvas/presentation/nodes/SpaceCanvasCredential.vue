<script setup lang="ts">
import { Eye, EyeOff, KeyRound, LockKeyhole, Pencil } from "@lucide/vue";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../../shared/ui/UiDialog.vue";
import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../../shared/ui/UiInput.vue";
import { useCredentialEditor } from "../../../credentials/use-credential-editor";
import { useNodeDraft } from "../../../inspection/presentation/use-node-draft";
import type { SpaceCredentialPayload } from "../../../model/nodes/node-payload";

const props = defineProps<{
  id: string;
  data: SpaceCredentialPayload;
  interactive: boolean;
  persist: (nodeId: string, payload: SpaceCredentialPayload) => Promise<void>;
}>();
const { t } = useI18n();
const {
  open,
  loading,
  saving,
  reveal,
  title,
  secret,
  username,
  password,
  canSave,
  openEditor: openCredentialEditor,
  toggleReveal,
  save,
} = useCredentialEditor(props);
const nodeDraft = useNodeDraft(computed(() => props.id));
watch(open, (value) => {
  if (!value) nodeDraft.end();
});
function beginEditor() {
  if (nodeDraft.begin()) void openCredentialEditor();
}
const kindLabel = computed(() =>
  props.data.credentialKind === "api-key"
    ? t("spaces.credentials.apiKey")
    : t("spaces.credentials.accountPassword"),
);
</script>

<template>
  <div class="pointer-events-none flex size-full min-w-0 items-center gap-3 px-3">
    <KeyRound
      v-if="data.credentialKind === 'api-key'"
      class="size-3.5 shrink-0 text-fg-secondary"
      :stroke-width="1.6"
    />
    <LockKeyhole v-else class="size-3.5 shrink-0 text-fg-secondary" :stroke-width="1.6" />
    <span class="grid min-w-0 flex-1 gap-0.5">
      <strong class="block truncate text-label font-medium">{{ data.title }}</strong>
      <span class="block truncate text-caption text-fg-secondary">{{ kindLabel }}</span>
    </span>
    <UiIconButton
      class="shrink-0"
      :class="interactive && 'pointer-events-auto'"
      :disabled="!interactive || !nodeDraft.available.value"
      :label="t('spaces.credentials.edit')"
      size="sm"
      @pointerdown.stop
      @dblclick.stop
      @click.stop="beginEditor"
    >
      <Pencil class="size-3.5" :stroke-width="1.5" />
    </UiIconButton>
  </div>

  <UiDialog
    :open="open"
    :title="t('spaces.credentials.edit')"
    class="bg-surface-canvas"
    @update:open="
      (value) => {
        if (!saving) open = value;
      }
    "
  >
    <div class="grid gap-3">
      <label class="grid gap-1.5 text-caption text-fg-secondary">
        <span>{{ t("spaces.credentials.title") }}</span>
        <UiInput v-model="title" :disabled="loading || saving" />
      </label>
      <label
        v-if="data.credentialKind === 'account-password'"
        class="grid gap-1.5 text-caption text-fg-secondary"
      >
        <span>{{ t("spaces.credentials.username") }}</span>
        <UiInput v-model="username" :disabled="loading || saving" />
      </label>
      <label class="grid gap-1.5 text-caption text-fg-secondary">
        <span>{{
          data.credentialKind === "api-key"
            ? t("spaces.credentials.apiKey")
            : t("spaces.credentials.password")
        }}</span>
        <span class="flex min-w-0 items-center gap-1.5">
          <UiInput
            v-if="data.credentialKind === 'api-key'"
            v-model="secret"
            class="flex-1"
            :disabled="loading || saving"
            :type="reveal ? 'text' : 'password'"
          />
          <UiInput
            v-else
            v-model="password"
            class="flex-1"
            :disabled="loading || saving"
            :type="reveal ? 'text' : 'password'"
          />
          <UiIconButton
            :disabled="loading || saving"
            :label="reveal ? t('common.hideSecret') : t('common.showSecret')"
            @click="toggleReveal()"
          >
            <EyeOff v-if="reveal" class="size-3.5" />
            <Eye v-else class="size-3.5" />
          </UiIconButton>
        </span>
      </label>
    </div>
    <template #footer>
      <UiButton variant="ghost" :disabled="saving" @click="open = false">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" :disabled="!canSave" @click="save">{{ t("common.save") }}</UiButton>
    </template>
  </UiDialog>
</template>
