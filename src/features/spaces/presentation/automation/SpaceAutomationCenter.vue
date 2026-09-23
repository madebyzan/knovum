<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useAutomationCenter } from "../../automation/use-automation-center";
import SpaceAutomationHistory from "./SpaceAutomationHistory.vue";

const { t } = useI18n();
const {
  error,
  form,
  misfireOptions,
  occurrences,
  overlapOptions,
  previewSchedule,
  remove,
  renew,
  save,
  saving,
  schedulePreview,
  sourceLabel,
  sourceOptions,
  test: testTrigger,
  toggle,
  triggers,
  workflowOptions,
} = useAutomationCenter();
</script>

<template>
  <section class="flex h-full min-h-0 flex-col bg-panel">
    <form class="space-y-2 border-b border-divider p-3" @submit.prevent="save">
      <UiInput v-model="form.name" :placeholder="t('spaces.assistant.automationName')" />
      <UiSelect v-model="form.workflow" :label="t('spaces.assistant.workflows')" :options="workflowOptions" />
      <UiSelect
        v-model="form.sourceKind"
        :label="t('spaces.assistant.triggerSourceLabel')"
        :options="sourceOptions"
      />
      <div class="grid grid-cols-2 gap-2">
        <template v-if="form.sourceKind === 'cron'">
          <UiInput v-model="form.schedule" :placeholder="t('spaces.assistant.cronSchedule')" />
          <UiInput v-model="form.timezone" :placeholder="t('spaces.assistant.timezone')" />
          <UiSelect
            v-model="form.misfirePolicy"
            :label="t('spaces.assistant.misfireLabel')"
            :options="misfireOptions"
          />
          <UiButton size="sm" variant="ghost" type="button" @click="previewSchedule">{{
            t("spaces.assistant.previewSchedule")
          }}</UiButton>
        </template>
        <template v-else>
          <UiInput v-model="form.eventKind" :placeholder="t('spaces.assistant.eventKind')" />
          <UiInput v-model="form.filterExpression" :placeholder="t('spaces.assistant.eventFilter')" />
        </template>
        <UiSelect
          v-model="form.overlapPolicy"
          :label="t('spaces.assistant.overlapLabel')"
          :options="overlapOptions"
        />
      </div>
      <UiButton
        type="submit"
        size="sm"
        variant="primary"
        :disabled="saving || !form.name.trim() || !form.workflow"
      >
        {{ t("spaces.assistant.createAutomation") }}
      </UiButton>
      <p v-if="error" class="text-meta text-danger">{{ error }}</p>
      <p v-if="schedulePreview.length" class="text-micro text-foreground-secondary">
        {{ schedulePreview.map((time) => new Date(time).toLocaleString()).join(" · ") }}
      </p>
    </form>
    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <article
        v-for="trigger in triggers"
        :key="trigger.id"
        class="mb-1 rounded-control border border-border/70 bg-raised p-2"
      >
        <div class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate text-label font-medium">{{ trigger.name }}</span>
          <UiBadge size="sm" :variant="trigger.enabled ? 'success' : 'neutral'">{{
            trigger.enabled ? t("spaces.assistant.enabled") : t("spaces.assistant.disabled")
          }}</UiBadge>
        </div>
        <p class="mt-1 text-micro text-foreground-tertiary">{{ sourceLabel(trigger) }}</p>
        <p v-if="trigger.validUntil <= Date.now()" class="mt-1 text-micro text-warning">
          {{ t("spaces.assistant.authorizationExpired") }}
        </p>
        <UiButton class="mt-1" size="sm" variant="ghost" @click="toggle(trigger)">
          {{ trigger.enabled ? t("spaces.assistant.pause") : t("spaces.assistant.resume") }}
        </UiButton>
        <UiButton class="mt-1" size="sm" variant="ghost" @click="testTrigger(trigger)">
          {{ t("spaces.assistant.testAutomation") }}
        </UiButton>
        <UiButton class="mt-1" size="sm" variant="ghost" @click="renew(trigger)">
          {{ t("spaces.assistant.renewAuthorization") }}
        </UiButton>
        <UiButton class="mt-1" size="sm" variant="dangerSubtle" @click="remove(trigger)">
          {{ t("common.delete") }}
        </UiButton>
      </article>
      <p v-if="triggers.length === 0" class="text-meta p-6 text-center text-foreground-tertiary">
        {{ t("spaces.assistant.automationEmpty") }}
      </p>
    </div>
    <SpaceAutomationHistory :occurrences="occurrences" />
  </section>
</template>
