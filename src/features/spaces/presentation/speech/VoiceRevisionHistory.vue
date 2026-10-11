<script setup lang="ts">
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import { VOICE_ENGINE_LABELS } from "../../speech/voice-options";
import VoiceAudition from "./VoiceAudition.vue";

const props = defineProps<{ profile: DeepReadonly<VoiceProfileHistory>; disabled: boolean }>();
const emit = defineEmits<{ dirty: [value: boolean] }>();
const { t } = useI18n();
const selected = shallowRef(props.profile.activeRevisionId);
watch(
  () => props.profile.activeRevisionId,
  (id) => {
    selected.value = id;
  },
);
const options = computed(() =>
  props.profile.revisions.map((revision) => ({
    value: revision.id,
    label: `v${revision.revision} · ${revision.language} · ${VOICE_ENGINE_LABELS[revision.engineId]} · ${revision.speed}×`,
  })),
);
</script>

<template>
  <section class="space-y-2">
    <h3 class="text-caption font-medium text-fg-secondary">
      {{ t("spaces.speech.voice.history") }} ({{ profile.revisions.length }})
    </h3>
    <UiSelect v-model="selected" size="sm" :label="t('spaces.speech.voice.history')" :options :disabled />
    <VoiceAudition
      :source="{ kind: 'revision', revisionId: selected }"
      :disabled
      @dirty="emit('dirty', $event)"
    />
  </section>
</template>
