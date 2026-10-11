<script setup lang="ts">
import { Save } from "@lucide/vue";
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import { activeVoiceProfileRevision } from "../voice-profile";
import VoiceRevisionHistory from "./VoiceRevisionHistory.vue";

const props = defineProps<{ profile: DeepReadonly<VoiceProfileHistory>; disabled: boolean }>();
const emit = defineEmits<{
  dirty: [value: boolean];
  rename: [name: string];
  revise: [language: string, speed: number];
}>();
const { t } = useI18n();
const name = shallowRef("");
const language = shallowRef("");
const speed = shallowRef<number>();
const LANGUAGE_OPTIONS = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
];
const activeRevision = computed(() => activeVoiceProfileRevision(props.profile));
const nameChanged = computed(() => name.value.trim() !== props.profile.name);
const fieldsDirty = computed(
  () =>
    nameChanged.value ||
    language.value !== activeRevision.value.language ||
    speed.value !== activeRevision.value.speed,
);
const auditionDirty = shallowRef(false);

watch(
  () => props.profile.name,
  (value) => (name.value = value),
  { immediate: true },
);
watch(
  () => props.profile.activeRevisionId,
  () => {
    language.value = activeRevision.value.language;
    speed.value = activeRevision.value.speed;
  },
  { immediate: true },
);
watch(fieldsDirty, reportDirty);

function reportDirty() {
  emit("dirty", fieldsDirty.value || auditionDirty.value);
}

function setAuditionDirty(value: boolean) {
  auditionDirty.value = value;
  reportDirty();
}
</script>

<template>
  <div class="grid gap-5 md:grid-cols-2">
    <div class="space-y-4">
      <section class="space-y-1.5">
        <label class="block text-caption font-medium text-fg-secondary">
          {{ t("spaces.speech.voice.name") }}
          <UiInput v-model="name" class="mt-1" size="sm" :disabled />
        </label>
        <div class="flex justify-end">
          <UiButton
            size="sm"
            variant="primary"
            :disabled="disabled || !name.trim() || !nameChanged"
            @click="emit('rename', name.trim())"
          >
            <Save class="size-3.5" />{{ t("common.save") }}
          </UiButton>
        </div>
      </section>
      <section v-if="!profile.archivedAt && profile.sourceKind !== 'preset'" class="space-y-1.5">
        <h3 class="text-caption font-medium text-fg-secondary">
          {{ t("spaces.speech.voice.revise") }}
        </h3>
        <div class="grid grid-cols-[minmax(0,1fr)_5rem_auto] items-end gap-2">
          <label class="space-y-1 text-caption text-fg-secondary">
            <span>{{ t("spaces.speech.project.language") }}</span>
            <UiSelect
              v-model="language"
              size="sm"
              :label="t('spaces.speech.project.language')"
              :options="LANGUAGE_OPTIONS"
              :disabled
            />
          </label>
          <label class="space-y-1 text-caption text-fg-secondary">
            <span>{{ t("spaces.speech.voice.speed") }}</span>
            <UiNumberField v-model="speed" size="sm" :min="0.5" :max="2" :step="0.05" :disabled />
          </label>
          <UiButton
            size="sm"
            variant="primary"
            :disabled="
              disabled ||
              speed === undefined ||
              (language === activeRevision.language && speed === activeRevision.speed)
            "
            @click="speed !== undefined && emit('revise', language, speed)"
          >
            {{ t("spaces.speech.voice.revise") }}
          </UiButton>
        </div>
      </section>
    </div>
    <VoiceRevisionHistory :profile :disabled @dirty="setAuditionDirty" />
  </div>
</template>
