<script setup lang="ts">
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import { LABEL_BY_VOICE_ENGINE } from "../voice-options";
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
const nameChanged = computed(() => name.value.trim() !== props.profile.name && name.value.trim().length > 0);
const canRevise = computed(() => !props.profile.archivedAt && props.profile.sourceKind !== "preset");
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
watch([nameChanged, auditionDirty], () => {
  emit("dirty", nameChanged.value || auditionDirty.value);
});

function commitRename() {
  const trimmed = name.value.trim();
  if (trimmed && trimmed !== props.profile.name) {
    emit("rename", trimmed);
  } else {
    name.value = props.profile.name;
  }
}

function cancelRename() {
  name.value = props.profile.name;
}

function submitRevision() {
  if (
    speed.value !== undefined &&
    (language.value !== activeRevision.value.language || speed.value !== activeRevision.value.speed)
  ) {
    emit("revise", language.value, speed.value);
  }
}

function setAuditionDirty(value: boolean) {
  auditionDirty.value = value;
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div class="w-full sm:max-w-xs">
        <label class="block space-y-1 text-caption font-medium text-fg-secondary">
          <span>{{ t("spaces.speech.voice.name") }}</span>
          <UiInput
            v-model="name"
            size="sm"
            :disabled
            @blur="commitRename"
            @keydown.enter.prevent="commitRename"
            @keydown.esc="cancelRename"
          />
        </label>
      </div>
      <div class="flex items-center gap-2">
        <span class="rounded bg-surface-muted px-2.5 py-1 text-caption font-medium text-fg-secondary">
          v{{ activeRevision.revision }} · {{ activeRevision.language }} ·
          {{ LABEL_BY_VOICE_ENGINE[activeRevision.engineId] }}
        </span>
      </div>
    </div>

    <VoiceRevisionHistory :profile :disabled @dirty="setAuditionDirty" />

    <div
      v-if="canRevise"
      class="space-y-2 rounded-lg border border-stroke-divider/70 bg-surface-muted/40 p-3"
    >
      <h4 class="text-caption font-medium text-fg-secondary">
        {{ t("spaces.speech.voice.revise") }}
      </h4>
      <div class="flex flex-wrap items-end gap-2">
        <label class="min-w-[7rem] space-y-1 text-caption text-fg-secondary">
          <span>{{ t("spaces.speech.project.language") }}</span>
          <UiSelect
            v-model="language"
            size="sm"
            :label="t('spaces.speech.project.language')"
            :options="LANGUAGE_OPTIONS"
            :disabled
          />
        </label>
        <label class="w-20 space-y-1 text-caption text-fg-secondary">
          <span>{{ t("spaces.speech.voice.speed") }}</span>
          <UiNumberField v-model="speed" size="sm" :min="0.5" :max="2" :step="0.05" :disabled />
        </label>
        <UiButton
          size="sm"
          variant="secondary"
          :disabled="
            disabled ||
            speed === undefined ||
            (language === activeRevision.language && speed === activeRevision.speed)
          "
          @click="submitRevision"
        >
          {{ t("spaces.speech.voice.revise") }}
        </UiButton>
      </div>
    </div>
  </div>
</template>
