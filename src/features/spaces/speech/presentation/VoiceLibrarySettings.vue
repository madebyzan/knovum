<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { onScopeDispose, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { useSettingsLeaveContext } from "../../../../foundation/lifecycle/use-settings-leave-context";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsHeaderActions from "../../../../shared/ui/UiSettingsHeaderActions.vue";
import type { CreatePresetVoiceRequest } from "../../contracts/generated/CreatePresetVoiceRequest";
import { useSpeechVoices } from "../use-speech-voices";
import VoiceCreation, { type VoiceCreationKind } from "./VoiceCreation.vue";
import VoiceProfileAdministration from "./VoiceProfileAdministration.vue";

const { t } = useI18n();
const editingTarget = shallowRef<string>();
const creationKind = shallowRef<VoiceCreationKind>("preset");
const draftDirty = shallowRef(false);
const { creating, errors, addPresetAsync } = useSpeechVoices();
let admittedPreset: Promise<boolean> | undefined;

async function confirmDiscard() {
  if (admittedPreset && !(await admittedPreset)) return false;
  if (!draftDirty.value) return true;
  return confirmAction({
    title: t("spaces.speech.library.discard.title"),
    description: t("spaces.speech.library.discard.description"),
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

async function selectCreationKind(kind: VoiceCreationKind) {
  if (!(await confirmDiscard())) return;
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
  <div data-testid="voice-library-settings" class="grid gap-5">
    <UiSettingsHeaderActions>
      <UiButton variant="secondary" :disabled="creating" @click="toggleCreation">
        <Plus class="size-4" />{{ t("spaces.speech.library.add") }}
      </UiButton>
    </UiSettingsHeaderActions>
    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("spaces.speech.library.description") }}
    </p>
    <VoiceProfileAdministration
      :expanded-id="editingTarget === 'new' ? undefined : editingTarget"
      :creating="editingTarget === 'new'"
      @dirty="draftDirty = $event"
      @toggle="toggleProfile"
    >
      <template #create>
        <VoiceCreation
          :kind="creationKind"
          :creating="creating"
          :errors="errors"
          @close="requestEditingTarget(undefined)"
          @create="createPreset"
          @created="finalizeEditing"
          @dirty="draftDirty = $event"
          @select="selectCreationKind"
        />
      </template>
    </VoiceProfileAdministration>
  </div>
</template>
