<script setup lang="ts">
import { FileOutput } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction, shouldShowAction } from "../../../../shared/contracts/action-availability";
import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import {
  DOCUMENT_CONVERSION_DEFINITIONS,
  type CreationIntentDecisions,
  type CreationIntentId,
} from "../catalog/creation-intent-definitions";

const props = defineProps<{ decisions: CreationIntentDecisions }>();
const emit = defineEmits<{ invoke: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const conversionActions = computed(() =>
  DOCUMENT_CONVERSION_DEFINITIONS.filter(({ id }) => shouldShowAction(props.decisions[id])),
);

function unavailableReason(actionId: CreationIntentId) {
  const decision = props.decisions[actionId];
  if (decision.state !== "blocked") return undefined;
  if (decision.reason === "needs-document-recognition")
    return t("spaces.creation.document.conversion.recognitionRequired");
  if (decision.reason === "active-node-operation") return t("spaces.creation.shared.finishNodeOperation");
  return undefined;
}
</script>

<template>
  <UiDropdownMenuItem
    v-for="action in conversionActions"
    :key="action.id"
    :data-conversion-action="action.id"
    :disabled="!canRunAction(decisions[action.id])"
    :title="unavailableReason(action.id)"
    class="h-auto items-start py-1.5"
    @select="emit('invoke', action.id)"
  >
    <FileOutput class="mt-0.5 size-3.5 shrink-0" />
    <span class="min-w-0">
      <span class="block">{{
        t(`spaces.creation.document.conversion.formats.${action.documentFormat}`)
      }}</span>
      <span class="block max-w-64 text-caption-sm leading-snug text-fg-tertiary">
        {{ t(`spaces.creation.document.conversion.expectations.${action.documentFormat}`) }}
      </span>
    </span>
  </UiDropdownMenuItem>
</template>
