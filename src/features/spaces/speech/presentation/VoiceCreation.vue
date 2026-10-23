<script setup lang="ts">
import { X } from "@lucide/vue";
import { TabsContent, TabsRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { CreatePresetVoiceRequest } from "../../contracts/generated/CreatePresetVoiceRequest";
import type { VoiceCandidateSourceKind } from "../use-voice-candidate-form";
import SpeechSegmentedTabs from "./SpeechSegmentedTabs.vue";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";
import VoiceProfileForm from "./VoiceProfileForm.vue";

export type VoiceCreationKind = "preset" | VoiceCandidateSourceKind;

const props = defineProps<{
  kind: VoiceCreationKind;
  creating: boolean;
  errors: readonly string[];
}>();
const emit = defineEmits<{
  close: [];
  create: [input: CreatePresetVoiceRequest];
  created: [];
  dirty: [value: boolean];
  select: [kind: VoiceCreationKind];
}>();
const { t } = useI18n();
// Where a voice comes from decides the whole form, so it leads the panel as a visible choice.
const sources = computed(() => [
  { value: "preset" as const, label: t("spaces.speech.library.presets") },
  { value: "reference" as const, label: t("spaces.speech.voice.clone.fromReference") },
  { value: "designed" as const, label: t("spaces.speech.voice.design.action") },
]);

function select(value: string) {
  if (value !== props.kind) emit("select", value as VoiceCreationKind);
}
</script>

<template>
  <TabsRoot :model-value="kind" class="space-y-4" @update:model-value="select">
    <header class="flex items-center gap-3">
      <h3 class="min-w-0 flex-1 text-label font-medium text-fg-primary">
        {{ t("spaces.speech.library.add") }}
      </h3>
      <UiIconButton :label="t('common.close')" size="sm" @click="emit('close')">
        <X class="size-3.5" />
      </UiIconButton>
    </header>
    <SpeechSegmentedTabs :label="t('spaces.speech.voice.mode')" :items="sources" />
    <div v-if="errors.length" class="space-y-1" role="alert">
      <p v-for="(message, index) in errors" :key="index" class="text-caption text-fg-danger">
        {{ message }}
      </p>
    </div>
    <TabsContent v-for="source in sources" :key="source.value" :value="source.value" class="outline-none">
      <VoiceProfileForm
        v-if="source.value === 'preset'"
        :disabled="creating"
        @create="emit('create', $event)"
        @dirty="emit('dirty', $event)"
      />
      <VoiceCandidateForm
        v-else
        :source-kind="source.value"
        @created="emit('created')"
        @dirty="emit('dirty', $event)"
      />
    </TabsContent>
  </TabsRoot>
</template>
