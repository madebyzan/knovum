<script setup lang="ts">
import { Archive, Copy, RotateCcw, Save, Trash2 } from "@lucide/vue";
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import { voiceEngineLabels } from "../../speech/voice-options";
import VoiceRevisionHistory from "./VoiceRevisionHistory.vue";

const props = defineProps<{
  profile: DeepReadonly<VoiceProfileHistory>;
  targets: readonly { value: string; label: string }[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  rename: [name: string];
  revise: [language: string, speed: number];
  archive: [];
  unarchive: [];
  remove: [];
  copy: [spaceId: string, name: string];
}>();
const { t } = useI18n();
const name = shallowRef("");
const language = shallowRef("");
const targetSpaceId = shallowRef("");
const copyName = shallowRef("");
const languageOptions = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
];
const activeRevision = computed(() =>
  props.profile.revisions.find((revision) => revision.id === props.profile.activeRevisionId),
);

watch(
  [() => props.profile.updatedAt, () => props.profile.activeRevisionId],
  () => {
    name.value = props.profile.name;
    copyName.value = props.profile.name;
    language.value = activeRevision.value?.language ?? "";
    targetSpaceId.value ||= props.targets[0]?.value ?? "";
  },
  { immediate: true },
);

async function remove() {
  if (
    await confirmAction({
      title: t("spaces.speech.voice.delete.title"),
      description: t("spaces.speech.voice.delete.description"),
      confirmLabel: t("common.delete"),
      danger: true,
    })
  )
    emit("remove");
}

function archiveLabel() {
  return props.profile.archivedAt ? t("common.restore") : t("common.archive");
}
</script>

<template>
  <article class="space-y-2 rounded-md border border-stroke-divider p-2">
    <div class="flex gap-2">
      <UiInput
        v-model="name"
        class="min-w-0 flex-1"
        size="sm"
        :aria-label="t('spaces.speech.voice.name')"
        :disabled="disabled"
      />
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled || !name.trim() || name.trim() === profile.name"
        @click="emit('rename', name.trim())"
        ><Save class="size-3.5" />{{ t("common.save") }}</UiButton
      >
    </div>
    <p class="text-caption text-fg-secondary">
      {{ t(`spaces.speech.voice.origins.${profile.sourceKind}`) }} ·
      {{ activeRevision && voiceEngineLabels[activeRevision.engineId] }} ·
      {{
        t(
          "spaces.speech.voice.usage",
          { count: profile.referencedSegmentCount },
          profile.referencedSegmentCount,
        )
      }}
    </p>
    <div v-if="!profile.archivedAt && profile.sourceKind !== 'preset'" class="flex gap-2">
      <UiSelect
        v-model="language"
        class="min-w-28"
        size="sm"
        :label="t('spaces.speech.project.language')"
        :options="languageOptions"
        :disabled="disabled"
      />
      <UiButton
        size="sm"
        :disabled="disabled || language === activeRevision?.language"
        @click="emit('revise', language, 1)"
        >{{ t("spaces.speech.voice.revise") }}</UiButton
      >
    </div>
    <VoiceRevisionHistory :profile :disabled />
    <div v-if="targets.length" class="flex gap-2">
      <UiSelect
        v-model="targetSpaceId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.project.targetSpace')"
        :options="targets"
        :disabled="disabled"
      />
      <UiInput
        v-model="copyName"
        class="min-w-0 flex-1"
        size="sm"
        :aria-label="t('spaces.speech.voice.name')"
        :disabled="disabled"
      />
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled || !targetSpaceId || !copyName.trim()"
        @click="emit('copy', targetSpaceId, copyName.trim())"
        ><Copy class="size-3.5" />{{ t("common.copy") }}</UiButton
      >
    </div>
    <div class="flex gap-2">
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled"
        @click="profile.archivedAt ? emit('unarchive') : emit('archive')"
      >
        <RotateCcw v-if="profile.archivedAt" class="size-3.5" />
        <Archive v-else class="size-3.5" />
        {{ archiveLabel() }}
      </UiButton>
      <UiButton
        v-if="profile.archivedAt"
        size="sm"
        variant="dangerSubtle"
        :disabled="disabled || profile.referencedSegmentCount > 0"
        @click="remove"
        ><Trash2 class="size-3.5" />{{ t("common.delete") }}</UiButton
      >
    </div>
  </article>
</template>
