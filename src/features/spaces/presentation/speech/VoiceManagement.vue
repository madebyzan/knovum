<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import { useVoiceManagement } from "../../speech/use-voice-management";
import VoiceProfileEditor from "./VoiceProfileEditor.vue";

const props = defineProps<{ spaceId: string }>();
const emit = defineEmits<{ changed: [] }>();
const { t } = useI18n();
const store = useSpaceStore();
const management = useVoiceManagement(
  () => props.spaceId,
  () => emit("changed"),
);
const targets = computed(() =>
  store.spaces
    .filter((space) => space.id !== props.spaceId && space.archivedAt == null)
    .map((space) => ({ value: space.id, label: space.title })),
);
defineExpose({ reload: management.reload });
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-4">
    <h2 class="text-label font-medium text-fg-primary">{{ t("spaces.speech.library.title") }}</h2>
    <p v-if="management.error.value" role="alert" class="text-caption text-fg-danger">
      {{ management.error.value }}
    </p>
    <p v-if="management.profiles.value.length === 0" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.library.empty") }}
    </p>
    <VoiceProfileEditor
      v-for="profile in management.profiles.value"
      :key="profile.id"
      :profile="profile"
      :targets="targets"
      :disabled="Boolean(management.busyProfileId.value)"
      @rename="management.rename(profile, $event)"
      @revise="(language, speed) => management.revise(profile, language, speed)"
      @archive="management.archive(profile)"
      @unarchive="management.unarchive(profile)"
      @remove="management.remove(profile)"
      @copy="(spaceId, name) => management.copy(profile, spaceId, name)"
    />
  </section>
</template>
