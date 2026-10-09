<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { extensionAvailabilityQuery, pluginQuery } from "../../../foundation/extensions/extension-query";
const props = defineProps<{
  kind: "skills" | "connections";
  extensionId: string;
  enabled: boolean;
  pluginId?: string | null | undefined;
}>();
const availability = useQuery(extensionAvailabilityQuery);
const plugins = useQuery(pluginQuery);
const error = computed(() => availability.error.value || plugins.error.value);
const isPluginPaused = computed(
  () =>
    Boolean(props.pluginId) &&
    !plugins.data.value?.some((plugin) => plugin.id === props.pluginId && plugin.enabled),
);
const status = computed(() => {
  if (error.value) return "failed";
  if (availability.isPending.value || (props.pluginId && plugins.isPending.value)) return "checking";
  if (isPluginPaused.value) return "pluginPaused";
  if (!props.enabled) return "disabled";
  return availability.data.value?.[props.kind].some((extension) => extension.id === props.extensionId)
    ? "available"
    : "unavailable";
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
