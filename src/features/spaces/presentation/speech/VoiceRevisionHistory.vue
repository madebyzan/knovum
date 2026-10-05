<script setup lang="ts">
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import VoiceAudition from "./VoiceAudition.vue";

const props = defineProps<{ profile: DeepReadonly<VoiceProfileHistory>; disabled: boolean }>();
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
    label: `v${revision.revision} · ${revision.language} · ${revision.speed}× · ${revision.modelSpecId}`,
  })),
);
</script>

<template>
  <details>
    <summary class="cursor-pointer text-caption text-fg-secondary">
      {{ t("spaces.speech.voiceHistory") }} ({{ profile.revisions.length }})
    </summary>
    <div class="mt-2 space-y-2">
      <UiSelect
        v-model="selected"
        size="sm"
        :label="t('spaces.speech.voiceHistory')"
        :options="options"
        :disabled
      />
      <VoiceAudition :source="{ kind: 'revision', revisionId: selected }" :disabled />
    </div>
  </details>
</template>
