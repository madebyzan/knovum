<script setup lang="ts">
import { FileOutput } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction, showsAction } from "../../../../shared/contracts/action-availability";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import {
  DOCUMENT_CONVERSION_DEFINITIONS,
  type CreationIntentDecisions,
  type CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";

const props = defineProps<{ decisions: CreationIntentDecisions }>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const conversionActions = computed(() =>
  DOCUMENT_CONVERSION_DEFINITIONS.filter(({ id }) => showsAction(props.decisions[id])),
);
</script>

<template>
  <p class="max-w-64 px-2 py-1.5 text-caption text-fg-tertiary">
    {{ t("spaces.creation.document.convert.help") }}
  </p>
  <UiDropdownMenuItem
    v-for="action in conversionActions"
    :key="action.id"
    :data-conversion-action="action.id"
    :disabled="!canRunAction(decisions[action.id])"
    :title="!canRunAction(decisions[action.id]) ? t('spaces.creation.shared.finishNodeOperation') : undefined"
    @select="emit('invoke', action.id)"
  >
    <FileOutput class="size-3.5" />
    {{ t(`spaces.creation.document.convert.formats.${action.documentFormat}`) }}
  </UiDropdownMenuItem>
</template>
