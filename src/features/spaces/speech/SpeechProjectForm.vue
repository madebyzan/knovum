<script setup lang="ts">
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import type { CreateSpeechProjectInput } from "./contracts";

const emit = defineEmits<{ create: [input: Omit<CreateSpeechProjectInput, "spaceId">] }>();
const { t } = useI18n();
const title = shallowRef("");

function submit() {
  if (!title.value.trim()) return;
  emit("create", {
    title: title.value.trim(),
    projectType: "short_dubbing",
    outputConfig: { format: "wav" },
  });
  title.value = "";
}
</script>

<template>
  <form class="space-y-2" @submit.prevent="submit">
    <UiInput v-model="title" size="sm" :placeholder="t('spaces.speech.projectName')" />
    <UiButton class="w-full" size="sm" variant="primary" type="submit">
      {{ t("spaces.speech.createProject") }} · {{ t("spaces.speech.types.shortDubbing") }}
    </UiButton>
  </form>
</template>
