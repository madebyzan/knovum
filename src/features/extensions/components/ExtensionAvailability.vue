<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { extensionAvailabilityQuery } from "../../../foundation/extensions/extension-query";
const props = defineProps<{
  kind: "skills" | "connections";
  extensionId: string;
  enabled: boolean;
  pluginId?: string | null | undefined;
}>();
const availability = useQuery(extensionAvailabilityQuery);
const error = availability.error;
const isAvailable = computed(() => {
  const current = availability.data.value;
  if (!current) return false;
  return props.pluginId
    ? current.plugins.some((plugin) => plugin.id === props.pluginId)
    : current[props.kind].some((extension) => extension.id === props.extensionId);
});
const status = computed(() => {
  if (error.value) return "failed";
  if (availability.isPending.value) return "checking";
  if (!props.enabled) return props.pluginId ? "pluginPaused" : "disabled";
  return isAvailable.value ? "available" : "unavailable";
});
</script>
<template>
  <span
    class="block text-caption"
    :class="status === 'failed' || status === 'unavailable' ? 'text-fg-warning' : 'text-fg-tertiary'"
    :title="error ? getErrorMessage(error) : undefined"
    :role="error ? 'alert' : undefined"
    >{{ $t(`extensions.overview.status.${status}`) }}</span
  >
</template>
