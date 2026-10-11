<script setup lang="ts">
import { shallowRef } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { updateMcpConnection } from "../../../../foundation/extensions/connection-operations";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import { useSettingsLeaveContext } from "../../../../shared/contracts/settings-leave";
import UiSwitch from "../../../../shared/ui/UiSwitch.vue";
import { useExtensionChanges } from "../../use-extension-changes";
const props = defineProps<{ connection: McpConnection; configured: boolean; disabled?: boolean }>();
const leaveCoordinator = useSettingsLeaveContext();
const changes = useExtensionChanges();
const navigationError = shallowRef("");
async function toggle() {
  navigationError.value = "";
  try {
    await leaveCoordinator.requestLeave();
  } catch (cause) {
    if (!isApplicationLeaveCancelled(cause)) navigationError.value = getErrorMessage(cause);
    return;
  }
  void changes
    .mutateAsync(async () => {
      await updateMcpConnection({ ...props.connection, enabled: !props.connection.enabled });
    })
    .catch(() => {});
}
</script>
<template>
  <div class="flex flex-col items-end gap-1">
    <UiSwitch
      :model-value="connection.enabled"
      :disabled="disabled || changes.isPending.value || (!configured && !connection.enabled)"
      :label="
        $t(connection.enabled ? 'common.disable' : 'common.enable', {
          name: connection.name,
        })
      "
      @update:model-value="toggle"
    />
    <p
      v-if="navigationError || changes.error.value"
      role="alert"
      class="max-w-48 text-caption text-fg-danger"
    >
      {{ navigationError || getErrorMessage(changes.error.value) }}
    </p>
  </div>
</template>
