<script setup lang="ts">
import { Copy, KeyRound, User } from "@lucide/vue";
import { writeText } from "tauri-plugin-clipboard-api";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { readSpaceCredentialField, type SpaceCredentialField } from "../../../credentials/credential-actions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceCredentialPayload } from "../../../model/nodes/node-payload";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";

const COPIED_LABEL_KEY: Record<SpaceCredentialField, string> = {
  "api-key": "spaces.credentials.copiedApiKey",
  username: "spaces.credentials.copiedUsername",
  password: "spaces.credentials.copiedPassword",
};

const props = defineProps<{
  node: SpaceCanvasNode<SpaceCredentialPayload>;
  compact?: boolean | undefined;
  menuItem?: boolean | undefined;
}>();
const { t } = useI18n();
const copying = shallowRef(false);
const configured = computed(() => Boolean(props.node.data.credentialConfigured));
const unavailableReason = computed(() =>
  configured.value ? undefined : t("spaces.credentials.notConfigured"),
);

async function copyField(field: SpaceCredentialField) {
  if (!configured.value || copying.value) return;
  copying.value = true;
  try {
    const value = await readSpaceCredentialField(props.node.id, field);
    if (!value) {
      toast.info(t("spaces.credentials.notConfigured"));
      return;
    }
    await writeText(value);
    toast.success(t(COPIED_LABEL_KEY[field]));
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    copying.value = false;
  }
}
</script>

<template>
  <SpaceCanvasActionTrigger
    v-if="node.data.credentialKind === 'api-key'"
    :label="t('spaces.credentials.copyApiKey')"
    :icon="Copy"
    :compact="compact"
    :menu-item="menuItem"
    :disabled="!configured || copying"
    :unavailable-reason="unavailableReason"
    @invoke="void copyField('api-key')"
  />
  <template v-else>
    <SpaceCanvasActionTrigger
      :label="t('spaces.credentials.copyUsername')"
      :icon="User"
      :compact="compact"
      :menu-item="menuItem"
      :disabled="!configured || copying"
      :unavailable-reason="unavailableReason"
      @invoke="void copyField('username')"
    />
    <SpaceCanvasActionTrigger
      :label="t('spaces.credentials.copyPassword')"
      :icon="KeyRound"
      :compact="compact"
      :menu-item="menuItem"
      :disabled="!configured || copying"
      :unavailable-reason="unavailableReason"
      @invoke="void copyField('password')"
    />
  </template>
</template>
