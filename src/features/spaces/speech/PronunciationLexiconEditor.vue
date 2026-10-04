<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import type { PronunciationEntry } from "./use-pronunciation-lexicon";

const entries = defineModel<PronunciationEntry[]>({ required: true });
defineProps<{ disabled: boolean; valid: boolean }>();
const { t } = useI18n();

function addEntry() {
  entries.value = [...entries.value, { id: crypto.randomUUID(), word: "", pronunciation: "" }];
}

function removeEntry(id: string) {
  entries.value = entries.value.filter((entry) => entry.id !== id);
}

function updateEntry(id: string, patch: Partial<Pick<PronunciationEntry, "word" | "pronunciation">>) {
  entries.value = entries.value.map((entry) => (entry.id === id ? Object.assign({}, entry, patch) : entry));
}
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-divider p-2">
    <div class="flex items-center justify-between gap-2">
      <div>
        <h3 class="text-caption font-medium text-fg-secondary">
          {{ t("spaces.speech.pronunciationLexicon") }}
        </h3>
        <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.speech.pronunciationHint") }}</p>
      </div>
      <UiButton size="sm" variant="ghost" :disabled="disabled || entries.length >= 1000" @click="addEntry">
        <Plus class="size-3.5" />{{ t("spaces.speech.addPronunciation") }}
      </UiButton>
    </div>
    <div v-for="entry in entries" :key="entry.id" class="flex items-center gap-1">
      <UiInput
        :model-value="entry.word"
        class="min-w-0 flex-1"
        size="sm"
        :disabled="disabled"
        :placeholder="t('spaces.speech.pronunciationWord')"
        @update:model-value="updateEntry(entry.id, { word: $event })"
      />
      <UiInput
        :model-value="entry.pronunciation"
        class="min-w-0 flex-1"
        size="sm"
        :disabled="disabled"
        :placeholder="t('spaces.speech.pronunciationReading')"
        @update:model-value="updateEntry(entry.id, { pronunciation: $event })"
      />
      <UiIconButton
        size="sm"
        :disabled="disabled"
        :label="t('spaces.speech.removePronunciation')"
        @click="removeEntry(entry.id)"
      >
        <Trash2 class="size-3.5" />
      </UiIconButton>
    </div>
    <p v-if="!valid" class="text-caption-sm text-fg-danger">
      {{ t("spaces.speech.pronunciationInvalid") }}
    </p>
  </section>
</template>
