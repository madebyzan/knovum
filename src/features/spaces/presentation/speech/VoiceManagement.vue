<script setup lang="ts">
import { AudioLines } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import { useVoiceManagement } from "../../speech/use-voice-management";
import VoiceProfileConfiguration from "./VoiceProfileConfiguration.vue";

const props = defineProps<{ expandedId?: string | undefined; creating?: boolean | undefined }>();
const emit = defineEmits<{ dirty: [value: boolean]; toggle: [profileId: string] }>();
const { t } = useI18n();
const { reload, profiles, loading, busyProfileId, errors, rename, revise, archive, unarchive, remove } =
  useVoiceManagement();
</script>

<template>
  <UiSettingsSection>
    <div v-if="creating" class="border-b border-stroke-divider bg-surface-muted px-4 py-4">
      <slot name="create" />
    </div>
    <div
      v-if="loading"
      class="flex items-center gap-2 px-4 py-6 text-caption text-fg-secondary"
      role="status"
    >
      <UiSpinner size="sm" />{{ t("common.loading") }}
    </div>
    <div
      v-else-if="errors.length && profiles.length === 0"
      class="flex items-center gap-3 px-4 py-6 text-label text-fg-secondary"
      role="alert"
    >
      <div class="min-w-0 flex-1 space-y-1">
        <p v-for="(message, index) in errors" :key="index">{{ message }}</p>
      </div>
      <UiButton size="sm" @click="reload()">{{ t("common.retry") }}</UiButton>
    </div>
    <div v-else-if="profiles.length" class="divide-y divide-stroke-divider">
      <VoiceProfileConfiguration
        v-for="profile in profiles"
        :key="profile.id"
        :profile
        :expanded="expandedId === profile.id"
        :disabled="Boolean(busyProfileId)"
        @dirty="emit('dirty', $event)"
        @toggle="emit('toggle', profile.id)"
        @rename="rename(profile, $event)"
        @revise="(language, speed) => revise(profile, language, speed)"
        @archive="archive(profile)"
        @unarchive="unarchive(profile)"
        @remove="remove(profile)"
      />
    </div>
    <div v-else class="px-4 py-6">
      <AudioLines class="size-4 text-fg-tertiary" :stroke-width="1.5" />
      <p class="mt-2 text-label text-fg-secondary">{{ t("spaces.speech.library.empty") }}</p>
    </div>
    <p
      v-if="errors.length && profiles.length"
      class="border-t border-stroke-divider px-4 py-3 text-caption text-fg-danger"
      role="alert"
    >
      <span v-for="(message, index) in errors" :key="index" class="block">{{ message }}</span>
    </p>
  </UiSettingsSection>
</template>
