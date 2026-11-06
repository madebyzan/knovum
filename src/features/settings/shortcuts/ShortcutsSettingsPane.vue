<script setup lang="ts">
import { AlertTriangle, RotateCcw } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import KeybindingRecorder from "./KeybindingRecorder.vue";
import ShortcutItemRow from "./ShortcutItemRow.vue";
import { useShortcutsEditor } from "./use-shortcuts-editor";

const { t } = useI18n();
const {
  query,
  recorderOpen,
  recordingTarget,
  shortcutsStore,
  allCommands,
  groupedCommands,
  openRecorder,
  handleSaveChord,
  handleReset,
  handleResetAll,
} = useShortcutsEditor();
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between gap-4">
      <UiSearchInput v-model="query" class="max-w-xs" :placeholder="t('shell.shortcuts.search')" />
      <UiButton variant="secondary" size="sm" @click="handleResetAll">
        <RotateCcw class="size-3.5" />
        {{ t("settings.shortcuts.resetAll") }}
      </UiButton>
    </div>

    <!-- Global Shortcuts Section -->
    <UiSettingsSection
      :title="t('settings.shortcuts.globalTitle')"
      :description="t('settings.shortcuts.globalDescription')"
    >
      <div
        v-if="shortcutsStore.dictationStatus && !shortcutsStore.dictationStatus.registered"
        class="mb-3 flex items-start gap-2.5 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-caption text-amber-500"
      >
        <AlertTriangle class="size-4 shrink-0 translate-y-0.5" />
        <div class="leading-relaxed">{{ t("settings.shortcuts.conflictWarning") }}</div>
      </div>

      <ShortcutItemRow
        id="dictation"
        :title="t('settings.shortcuts.dictationLabel')"
        :description="t('settings.shortcuts.dictationDescription')"
        :shortcut="shortcutsStore.dictationStatus?.shortcut ?? 'CommandOrControl+Shift+Space'"
        :customized="shortcutsStore.isCustomized('dictation')"
        :edit-label="t('settings.shortcuts.edit')"
        :reset-label="t('settings.shortcuts.reset')"
        :customized-label="t('settings.shortcuts.customized')"
        @edit="
          openRecorder(
            'dictation',
            t('settings.shortcuts.dictationLabel'),
            shortcutsStore.dictationStatus?.shortcut,
            true,
          )
        "
        @reset="handleReset('dictation', true)"
      />
    </UiSettingsSection>

    <!-- In-App Shortcuts Section -->
    <UiSettingsSection
      v-for="group in groupedCommands"
      :key="group.category"
      :title="group.title"
      :description="t('settings.shortcuts.inAppDescription')"
    >
      <ShortcutItemRow
        v-for="cmd in group.commands"
        :id="cmd.id"
        :key="cmd.id"
        :title="cmd.title"
        :shortcut="shortcutsStore.getEffectiveShortcut(cmd.id, cmd.shortcut)"
        :customized="shortcutsStore.isCustomized(cmd.id)"
        :edit-label="t('settings.shortcuts.edit')"
        :reset-label="t('settings.shortcuts.reset')"
        :customized-label="t('settings.shortcuts.customized')"
        @edit="openRecorder(cmd.id, cmd.title, shortcutsStore.getEffectiveShortcut(cmd.id, cmd.shortcut))"
        @reset="handleReset(cmd.id)"
      />
    </UiSettingsSection>

    <KeybindingRecorder
      v-if="recordingTarget"
      v-model:open="recorderOpen"
      :title="recordingTarget.title"
      :target-command-id="recordingTarget.id"
      :current-chord="recordingTarget.currentChord"
      :commands="allCommands"
      @save="handleSaveChord"
    />
  </div>
</template>
