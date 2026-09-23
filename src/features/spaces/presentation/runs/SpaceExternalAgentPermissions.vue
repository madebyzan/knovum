<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { decideExternalAgentPermission, type RunEvent } from "../../runs/run-client";

const props = defineProps<{ events: RunEvent[] }>();
const emit = defineEmits<{ resolved: [] }>();
const { t } = useI18n();
const resolvedIds = shallowRef<string[]>([]);
const pending = computed(() =>
  props.events.filter(
    (event) =>
      event.kind === "external_agent_permission_requested" &&
      !resolvedIds.value.includes(payload(event).requestId),
  ),
);

function payload(event: RunEvent) {
  return event.payload as {
    sessionId: string;
    requestId: string;
    title: string;
    options: Array<{ id: string; label: string; kind: string }>;
  };
}

async function decide(event: RunEvent, optionId: string | null) {
  const request = payload(event);
  await decideExternalAgentPermission(request.sessionId, request.requestId, optionId);
  resolvedIds.value = [...resolvedIds.value, request.requestId];
  emit("resolved");
}
</script>

<template>
  <article
    v-for="event in pending"
    :key="event.sequence"
    class="mb-2 rounded-control border border-warning/40 bg-raised p-2"
  >
    <p class="text-label font-medium">{{ payload(event).title }}</p>
    <div class="mt-2 flex flex-wrap gap-1">
      <UiButton
        v-for="option in payload(event).options"
        :key="option.id"
        size="sm"
        :variant="option.kind.startsWith('reject') ? 'dangerSubtle' : 'primary'"
        @click="decide(event, option.id)"
      >
        {{ option.label }}
      </UiButton>
      <UiButton size="sm" variant="ghost" @click="decide(event, null)">{{ t("common.cancel") }}</UiButton>
    </div>
  </article>
</template>
