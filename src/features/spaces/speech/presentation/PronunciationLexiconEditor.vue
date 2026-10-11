<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { PronunciationRule } from "../use-pronunciation-lexicon";

const entries = defineModel<PronunciationRule[]>({ required: true });
defineProps<{ disabled: boolean; valid: boolean }>();
const { t } = useI18n();

function addRule() {
  entries.value = [...entries.value, { id: crypto.randomUUID(), word: "", pronunciation: "" }];
}

function removeRule(ruleId: string) {
  entries.value = entries.value.filter((entry) => entry.id !== ruleId);
}

function updateRule(ruleId: string, patch: Partial<Pick<PronunciationRule, "word" | "pronunciation">>) {
  entries.value = entries.value.map((entry) =>
    entry.id === ruleId ? Object.assign({}, entry, patch) : entry,
  );
}
</script>

<template>
  <section class="space-y-2">
    <div class="flex items-center justify-between gap-2">
      <div class="min-w-0 flex-1">
        <h3 class="text-caption font-medium text-fg-secondary">
          {{ t("spaces.speech.audiobook.lexicon.title") }}
        </h3>
        <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.speech.audiobook.lexicon.hint") }}</p>
      </div>
      <UiButton size="sm" variant="ghost" :disabled="disabled || entries.length >= 1000" @click="addRule">
        <Plus class="size-3.5" />{{ t("spaces.speech.audiobook.lexicon.add") }}
      </UiButton>
    </div>
    <div v-for="entry in entries" :key="entry.id" class="flex items-center gap-1">
      <UiInput
        :model-value="entry.word"
        class="min-w-0 flex-1"
        size="sm"
        :disabled="disabled"
        :placeholder="t('spaces.speech.audiobook.lexicon.word')"
        @update:model-value="updateRule(entry.id, { word: $event })"
      />
      <UiInput
        :model-value="entry.pronunciation"
        class="min-w-0 flex-1"
        size="sm"
        :disabled="disabled"
        :placeholder="t('spaces.speech.audiobook.lexicon.reading')"
        @update:model-value="updateRule(entry.id, { pronunciation: $event })"
      />
      <UiIconButton
        size="sm"
        :disabled="disabled"
        :label="t('spaces.speech.audiobook.lexicon.remove')"
        @click="removeRule(entry.id)"
      >
        <Trash2 class="size-3.5" />
      </UiIconButton>
    </div>
    <p v-if="!valid" class="text-caption-sm text-fg-danger">
      {{ t("spaces.speech.audiobook.lexicon.invalid") }}
    </p>
  </section>
</template>
