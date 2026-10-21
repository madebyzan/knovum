<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import { exportFormatLabel } from "../../export/naming";
import type { ExportTarget } from "../../export/target";
import { useExportFormats } from "../../export/use-export-formats";

const props = defineProps<{
  target: ExportTarget | undefined;
  side?: "top" | "bottom";
  align?: "start" | "center" | "end";
}>();
const emit = defineEmits<{ save: [format: string] }>();
const { t } = useI18n();
const open = ref(false);
const query = useExportFormats(
  computed(() => props.target),
  open,
);
const items = computed(() =>
  (query.data.value?.formats ?? []).map((format) => ({
    id: format,
    label: exportFormatLabel(format),
    onSelect: () => emit("save", format),
  })),
);
/** Why nothing can be chosen, said by whoever knows: the engine, or the listing itself. */
const unavailable = computed(() => {
  if (query.error.value) return getErrorMessage(query.error.value);
  if (query.isPending.value) return t("common.loading");
  if (items.value.length) return undefined;
  return query.data.value?.unavailable ?? t("spaces.importExport.export.noFormats");
});
</script>

<template>
  <UiDropdownMenu
    v-model:open="open"
    :side="side ?? 'bottom'"
    :align="align ?? 'end'"
    :side-offset="6"
    :items="items"
  >
    <template #trigger>
      <slot name="trigger" />
    </template>
    <p v-if="unavailable" class="px-2 py-1 text-caption text-fg-secondary">{{ unavailable }}</p>
  </UiDropdownMenu>
</template>
