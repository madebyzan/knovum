<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoLineDraft } from "../video-dubbing-draft";
import SpeechSettingsDisclosure from "./SpeechSettingsDisclosure.vue";

const props = defineProps<{
  lines: readonly VideoLineDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  bind: [speakerKey: string, revisionId: string];
  applyAll: [revisionId: string];
}>();
const { t } = useI18n();
const batchVoiceId = shallowRef("");

const speakers = computed(() => {
  const byKey = new Map<string, { count: number; revisions: Set<string> }>();
  for (const line of props.lines) {
    const key = line.speakerKey.trim();
    if (!key) continue;
    const entry = byKey.get(key) ?? { count: 0, revisions: new Set<string>() };
    entry.count += 1;
    entry.revisions.add(line.voiceRevisionId);
    byKey.set(key, entry);
  }
  return [...byKey].map(([key, { count, revisions }]) => ({
    key,
    count,
    revisionId: revisions.size === 1 ? [...revisions][0]! : "",
  }));
});

const summary = computed(() => {
  if (!speakers.value.length) return undefined;
  const uniqueIds = new Set(props.lines.map((line) => line.voiceRevisionId).filter(Boolean));
  const sharedId = uniqueIds.size === 1 ? [...uniqueIds][0] : undefined;
  const sharedLabel = sharedId
    ? props.voiceOptions.find((option) => option.value === sharedId)?.label
    : undefined;
  return sharedLabel ? `${speakers.value.length} · ${sharedLabel}` : String(speakers.value.length);
});
</script>

<template>
  <SpeechSettingsDisclosure
    v-if="speakers.length"
    :title="t('spaces.speech.project.roleBindings.label')"
    :summary="summary"
  >
    <p class="text-caption-sm text-fg-tertiary">
      {{ t("spaces.speech.project.roleBindings.hint") }}
    </p>
    <div
      v-if="lines.length > 1 && speakers.length > 1"
      class="flex items-center gap-1.5 rounded-md bg-surface-control px-2 py-1.5 text-caption"
    >
      <span class="shrink-0 font-medium whitespace-nowrap text-fg-secondary">
        {{ t("spaces.speech.videoDubbing.batchVoiceLabel") }}
      </span>
      <UiSelect
        v-model="batchVoiceId"
        size="sm"
        class="min-w-0 flex-1"
        :options="voiceOptions"
        :label="t('spaces.speech.voice.label')"
        :placeholder="t('spaces.speech.library.selection.placeholder')"
        :disabled
      />
      <UiButton
        size="sm"
        variant="secondary"
        class="shrink-0"
        :disabled="disabled || !batchVoiceId"
        @click="batchVoiceId && emit('applyAll', batchVoiceId)"
      >
        {{ t("spaces.speech.videoDubbing.applyToAllLines") }}
      </UiButton>
    </div>
    <div
      class="max-h-52 divide-y divide-stroke-divider overflow-y-auto rounded-md border border-stroke-default bg-surface-raised"
    >
      <div v-for="speaker in speakers" :key="speaker.key" class="flex items-center gap-2 px-2 py-1">
        <span
          class="max-w-20 min-w-8 shrink-0 truncate text-caption font-medium text-fg-secondary"
          :title="speaker.key"
        >
          {{ speaker.key }}
        </span>
        <UiBadge size="sm" variant="neutral" class="shrink-0 tabular-nums">
          {{ speaker.count }}
        </UiBadge>
        <UiSelect
          class="min-w-0 flex-1 border-transparent bg-transparent shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus"
          :model-value="speaker.revisionId"
          size="sm"
          :options="voiceOptions"
          :label="t('spaces.speech.voice.label')"
          :placeholder="t('spaces.speech.videoDubbing.mixedVoiceBindings')"
          :disabled
          @update:model-value="emit('bind', speaker.key, $event)"
        />
      </div>
    </div>
  </SpeechSettingsDisclosure>
</template>
