<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useLocalResourcesQuery } from "../../../../foundation/runtime/resource-query";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { refreshEnrichmentJobs } from "../../enrichment/enrichment-jobs-query";
import { requestMediaDiarization } from "../../media/transcription";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{
  nodeId: string;
  revision: number;
  durationMs: number;
  speakerCount: number;
  disabled: boolean;
}>();
const { t } = useI18n();
const router = useRouter();
const store = useSpaceStore();
const resources = useLocalResourcesQuery();
const minimum = shallowRef("");
const maximum = shallowRef("");
const submitting = shallowRef(false);
const error = shallowRef<string>();
const installed = computed(() => resources.data.value?.models.diarization.verified === true);
const accessible = computed(
  () => installed.value || resources.data.value?.models.diarization.tokenConfigured === true,
);
const boundsValid = computed(
  () =>
    [minimum.value, maximum.value].every(
      (value) =>
        !value.trim() || (Number.isInteger(Number(value)) && Number(value) >= 1 && Number(value) <= 32),
    ) &&
    (!minimum.value.trim() || !maximum.value.trim() || Number(minimum.value) <= Number(maximum.value)),
);
const canStart = computed(() => accessible.value && boundsValid.value && props.durationMs <= 7_200_000);

async function start() {
  const spaceId = store.activeSpaceId;
  if (!spaceId || props.disabled || submitting.value || !canStart.value) return;
  submitting.value = true;
  error.value = undefined;
  try {
    await requestMediaDiarization({
      spaceId,
      nodeId: props.nodeId,
      expectedRevision: props.revision,
      minimumSpeakers: minimum.value ? Number(minimum.value) : null,
      maximumSpeakers: maximum.value ? Number(maximum.value) : null,
    });
    await refreshEnrichmentJobs();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <section class="mt-3 space-y-2 rounded-panel border border-stroke-divider p-2.5">
    <p class="text-caption font-medium text-fg-secondary">
      {{ t("spaces.transcript.speakers.title") }} · {{ speakerCount }}
    </p>
    <p v-if="!accessible" class="text-caption text-fg-tertiary">
      {{ t("spaces.transcript.speakers.needsModel") }}
    </p>
    <UiButton
      v-if="!accessible"
      size="sm"
      variant="ghost"
      @click="router.push({ path: '/settings', query: { pane: 'aiModels' } })"
      >{{ t("spaces.transcript.speakers.configureAccess") }}</UiButton
    >
    <p v-else-if="!installed" class="text-caption text-fg-tertiary">
      {{ t("spaces.transcript.speakers.firstUse") }}
    </p>
    <p v-if="durationMs > 7_200_000" class="text-caption text-fg-tertiary">
      {{ t("spaces.transcript.speakers.durationLimit") }}
    </p>
    <div class="grid grid-cols-2 gap-2">
      <label class="text-caption text-fg-secondary"
        >{{ t("spaces.transcript.speakers.minimum") }}
        <UiInput
          v-model="minimum"
          type="number"
          size="sm"
          min="1"
          max="32"
          :disabled="disabled || submitting || !accessible"
        />
      </label>
      <label class="text-caption text-fg-secondary"
        >{{ t("spaces.transcript.speakers.maximum") }}
        <UiInput
          v-model="maximum"
          type="number"
          size="sm"
          min="1"
          max="32"
          :disabled="disabled || submitting || !accessible"
        />
      </label>
    </div>
    <UiButton size="sm" :disabled="disabled || submitting || !canStart" @click="start">{{
      t("spaces.transcript.speakers.generate")
    }}</UiButton>
    <p v-if="error || resources.error.value" class="text-caption text-fg-danger" role="alert">
      {{ error ?? getErrorMessage(resources.error.value) }}
    </p>
  </section>
</template>
