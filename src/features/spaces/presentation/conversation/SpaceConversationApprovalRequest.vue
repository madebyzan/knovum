<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { ApprovalSummary } from "../../contracts/generated/ActionApproval";
import { captureActionLabels } from "../workflows/workflow-capture-labels";

const props = defineProps<{ approval: ApprovalSummary; busy: boolean }>();
const emit = defineEmits<{ decide: [approvalId: string, approved: boolean] }>();
const { t } = useI18n();
const actionName = computed(() => {
  const label = captureActionLabels[props.approval.actionId];
  return label ? t(`spaces.creation.captureActions.${label}`) : props.approval.actionId;
});
const parameters = computed(() => {
  const plan = props.approval.plan;
  if (!plan || typeof plan !== "object" || Array.isArray(plan)) return [];
  const value = plan.parameters;
  if (!value || typeof value !== "object" || Array.isArray(value)) return [];
  return Object.entries(value).map(([name, parameter]) => ({
    name,
    value: typeof parameter === "string" ? parameter : JSON.stringify(parameter),
  }));
});
</script>

<template>
  <section
    class="bg-warning-subtle mb-4 rounded-panel border border-warning/40 p-3 text-caption"
    role="group"
    :aria-label="t('spaces.assistant.approvalPlan')"
  >
    <h3 class="font-semibold text-foreground">
      {{ t("spaces.assistant.actionApprovalTitle", { action: actionName }) }}
    </h3>
    <p class="mt-1 text-foreground-secondary">{{ t("spaces.assistant.actionApprovalHint") }}</p>
    <dl v-if="parameters.length" class="mt-2 space-y-1 border-t border-warning/30 pt-2">
      <div
        v-for="parameter in parameters"
        :key="parameter.name"
        class="grid grid-cols-[minmax(0,7rem)_minmax(0,1fr)] gap-2"
      >
        <dt class="truncate text-foreground-tertiary">{{ parameter.name }}</dt>
        <dd class="wrap-break-word text-foreground select-text">{{ parameter.value }}</dd>
      </div>
    </dl>
    <details class="mt-2 text-foreground-secondary">
      <summary class="cursor-pointer">{{ t("spaces.assistant.approvalPlan") }}</summary>
      <pre class="mt-2 max-h-48 overflow-auto wrap-break-word whitespace-pre-wrap select-text">{{
        JSON.stringify(approval.plan, null, 2)
      }}</pre>
    </details>
    <div class="mt-3 flex justify-end gap-2">
      <UiButton size="sm" variant="secondary" :disabled="busy" @click="emit('decide', approval.id, false)">{{
        t("common.reject")
      }}</UiButton>
      <UiButton size="sm" variant="primary" :disabled="busy" @click="emit('decide', approval.id, true)">{{
        t("common.approve")
      }}</UiButton>
    </div>
  </section>
</template>
