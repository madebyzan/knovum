<script setup lang="ts">
import { Plus, X } from "@lucide/vue";
import { computed, onScopeDispose, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { useSettingsLeaveContext } from "../../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { CreatePresetVoiceRequest } from "../../contracts/generated/CreatePresetVoiceRequest";
import { useSpeechVoices } from "../../speech/use-speech-voices";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";
import VoiceManagement from "./VoiceManagement.vue";
import VoiceProfileForm from "./VoiceProfileForm.vue";

type CreationKind = "preset" | "reference" | "designed";

const { t } = useI18n();
const editingTarget = shallowRef<string>();
const creationKind = shallowRef<CreationKind>("preset");
const draftDirty = shallowRef(false);
const { creating, errors, addPresetAsync } = useSpeechVoices();
let admittedPreset: Promise<boolean> | undefined;
const creationOptions = computed(() => [
  { value: "preset" as const, label: t("spaces.speech.library.presets") },
  { value: "reference" as const, label: t("spaces.speech.voice.clone") },
  { value: "designed" as const, label: t("spaces.speech.voice.design") },
]);

async function confirmDiscard() {
  if (admittedPreset && !(await admittedPreset)) return false;
  if (!draftDirty.value) return true;
  return confirmAction({
    title: t("spaces.speech.library.discardTitle"),
    description: t("spaces.speech.library.discardDescription"),
    confirmLabel: t("common.discardChanges"),
    danger: true,
  });
}

function finalizeEditing() {
  editingTarget.value = undefined;
  draftDirty.value = false;
}

async function requestEditingTarget(target: string | undefined) {
  if (target === editingTarget.value || !(await confirmDiscard())) return false;
  finalizeEditing();
  editingTarget.value = target;
  return true;
}

function toggleProfile(profileId: string) {
  void requestEditingTarget(editingTarget.value === profileId ? undefined : profileId);
}

function toggleCreation() {
  void requestEditingTarget(editingTarget.value === "new" ? undefined : "new");
}

async function selectCreationKind(kind: CreationKind) {
  if (kind === creationKind.value || !(await confirmDiscard())) return;
  draftDirty.value = false;
  creationKind.value = kind;
}

function createPreset(input: CreatePresetVoiceRequest) {
  if (admittedPreset) return;
  admittedPreset = persistPreset(input);
}

async function persistPreset(input: CreatePresetVoiceRequest) {
  try {
    await addPresetAsync(input);
    finalizeEditing();
    return true;
  } catch {
    // The mutation exposes its error while the draft remains available.
    return false;
  } finally {
    admittedPreset = undefined;
  }
}

const unregisterLeaveParticipant = useSettingsLeaveContext().register({
  prepare: confirmDiscard,
  finalize: finalizeEditing,
});
onScopeDispose(unregisterLeaveParticipant);
</script>

<template>
  <section data-testid="voice-library-settings" class="space-y-4">
    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("spaces.speech.library.description") }}
    </p>
    <section class="min-w-0">
      <header class="mb-3 flex items-end justify-between gap-4 px-0.5">
        <div class="min-w-0">
          <h2 class="text-title-sm font-semibold tracking-title text-fg-primary">
            {{ t("spaces.speech.library.collectionTitle") }}
          </h2>
          <p class="mt-1 text-caption text-fg-secondary">{{ t("spaces.speech.library.manage") }}</p>
        </div>
        <UiButton variant="secondary" :disabled="creating" @click="toggleCreation">
          <Plus class="size-4" />{{ t("spaces.speech.voice.add") }}
        </UiButton>
      </header>
      <VoiceManagement
        :expanded-id="editingTarget === 'new' ? undefined : editingTarget"
        :creating="editingTarget === 'new'"
        @dirty="draftDirty = $event"
        @toggle="toggleProfile"
      >
        <template #create>
          <div class="space-y-4">
            <div class="flex items-center gap-3">
              <h3 class="min-w-0 flex-1 text-label font-medium text-fg-primary">
                {{ t("spaces.speech.library.addTitle") }}
              </h3>
              <UiSelect
                :model-value="creationKind"
                class="w-44 shrink-0"
                size="sm"
                :label="t('spaces.speech.voice.mode')"
                :options="creationOptions"
                @update:model-value="selectCreationKind"
              />
              <UiIconButton :label="t('common.close')" size="sm" @click="requestEditingTarget(undefined)">
                <X class="size-3.5" />
              </UiIconButton>
            </div>
            <div v-if="errors.length" class="space-y-1" role="alert">
              <p v-for="(message, index) in errors" :key="index" class="text-caption text-fg-danger">
                {{ message }}
              </p>
            </div>
            <VoiceProfileForm
              v-if="creationKind === 'preset'"
              :disabled="creating"
              @create="createPreset"
              @dirty="draftDirty = $event"
            />
            <VoiceCandidateForm
              v-else
              :key="creationKind"
              :source-kind="creationKind"
              @created="finalizeEditing"
              @dirty="draftDirty = $event"
            />
          </div>
        </template>
      </VoiceManagement>
    </section>
  </section>
</template>
