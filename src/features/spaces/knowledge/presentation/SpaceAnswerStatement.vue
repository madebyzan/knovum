<script setup lang="ts">
import { CircleCheck, CircleHelp, CircleX } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { StatementEvidence } from "../../contracts/generated/StatementEvidence";
import type { StatementSupport } from "../../contracts/generated/StatementSupport";
import type { StatementView } from "../verification/answer-verification-view";

/** One checked sentence: what the sources say about it, and the person's own verdict. */
const props = defineProps<{ view: StatementView; recordable: boolean }>();
const chosen = defineModel<boolean>("chosen", { required: true });
const emit = defineEmits<{ correct: [support: StatementSupport]; open: [evidence: StatementEvidence] }>();
const { t } = useI18n();

const BADGES = {
  supported: { variant: "success", label: "spaces.knowledge.verification.support.supported" },
  partial: { variant: "accent", label: "spaces.knowledge.verification.support.partial" },
  unsupported: { variant: "danger", label: "spaces.knowledge.verification.support.unsupported" },
  uncited: { variant: "neutral", label: "spaces.knowledge.verification.support.uncited" },
  "not-checkable": { variant: "neutral", label: "spaces.knowledge.verification.support.notCheckable" },
} as const satisfies Record<StatementSupport, { variant: string; label: string }>;
const evidence = computed(() => [
  ...(props.view.verdict?.evidence ?? []),
  ...(props.view.verdict?.possibleSources ?? []),
]);
</script>

<template>
  <li class="flex min-w-0 gap-2 py-2">
    <UiCheckbox
      v-if="recordable"
      v-model="chosen"
      class="mt-0.5"
      :label="t('spaces.knowledge.verification.choose', { statement: view.statement.text })"
    />
    <div class="min-w-0 flex-1 space-y-1">
      <p class="text-fg-primary">{{ view.statement.text }}</p>
      <div class="flex flex-wrap items-center gap-1.5">
        <UiBadge size="sm" :variant="BADGES[view.support].variant">{{
          t(BADGES[view.support].label)
        }}</UiBadge>
        <span v-if="view.corrected" class="text-caption-sm text-fg-tertiary">
          {{ t("spaces.knowledge.verification.corrected") }}
        </span>
        <span v-if="view.verdict?.reason && !view.corrected" class="text-caption text-fg-secondary">
          {{ view.verdict.reason }}
        </span>
      </div>
      <ul v-if="evidence.length" class="space-y-1">
        <li v-for="entry in evidence" :key="`${entry.citation}:${entry.span.quoteStart}`">
          <button
            type="button"
            class="w-full rounded-md px-1 py-0.5 text-left text-caption text-fg-secondary hover:bg-surface-control"
            @click="emit('open', entry)"
          >
            <span class="text-fg-tertiary tabular-nums">[{{ entry.citation }}]</span>
            “{{ entry.span.quote }}”
          </button>
        </li>
      </ul>
    </div>
    <div v-if="recordable" class="flex shrink-0 items-start gap-0.5">
      <UiIconButton
        :label="t('spaces.knowledge.verification.markSupported')"
        tooltip
        size="sm"
        :disabled="view.support === 'supported'"
        @click="emit('correct', 'supported')"
      >
        <CircleCheck class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.knowledge.verification.markUnsupported')"
        tooltip
        size="sm"
        :disabled="view.support === 'unsupported'"
        @click="emit('correct', 'unsupported')"
      >
        <CircleX class="size-3.5" />
      </UiIconButton>
    </div>
    <CircleHelp
      v-else
      class="mt-0.5 size-3.5 shrink-0 text-fg-tertiary"
      :aria-label="t('spaces.knowledge.verification.notCheckable')"
    />
  </li>
</template>
