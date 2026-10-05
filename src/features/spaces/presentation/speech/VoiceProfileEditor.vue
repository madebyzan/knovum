<script setup lang="ts">
import { Archive, Copy, RotateCcw, Save, Trash2 } from "@lucide/vue";
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
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
const speed = shallowRef("1");
const targetSpaceId = shallowRef("");
const copyName = shallowRef("");
const languageOptions = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
];
const activeRevision = computed(() =>
  props.profile.revisions.find((revision) => revision.id === props.profile.activeRevisionId),
);
const isSpeedValid = computed(() => {
  const value = Number(speed.value);
  return activeRevision.value?.engineId === "voxcpm2" ? value === 1 : value >= 0.5 && value <= 2;
});

watch(
  () => [props.profile.updatedAt, props.profile.activeRevisionId] as const,
  () => {
    name.value = props.profile.name;
    copyName.value = props.profile.name;
    language.value = activeRevision.value?.language ?? "";
    speed.value = String(activeRevision.value?.speed ?? 1);
    targetSpaceId.value ||= props.targets[0]?.value ?? "";
  },
  { immediate: true },
);

async function remove() {
  if (
    await confirmAction({
      title: t("spaces.speech.deleteVoice"),
      description: t("spaces.speech.deleteVoiceDescription"),
      confirmLabel: t("common.delete"),
      danger: true,
    })
  )
    emit("remove");
}

function archiveLabel() {
  return props.profile.archivedAt ? t("spaces.speech.restoreVoice") : t("spaces.speech.archiveVoice");
}
</script>

<template>
  <article class="space-y-2 rounded-md border border-stroke-divider p-2">
    <div class="flex gap-2">
      <UiInput v-model="name" class="min-w-0 flex-1" size="sm" :disabled="disabled" />
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled || !name.trim() || name.trim() === profile.name"
        @click="emit('rename', name.trim())"
        ><Save class="size-3.5" />{{ t("common.save") }}</UiButton
      >
    </div>
    <p class="text-caption text-fg-secondary">
      {{ profile.sourceKind }} · {{ activeRevision?.engineId }} · {{ profile.referencedSegmentCount }}
      {{ t("spaces.speech.voiceReferences") }}
    </p>
    <div v-if="!profile.archivedAt" class="flex gap-2">
      <UiSelect
        v-model="language"
        class="min-w-28"
        size="sm"
        :label="t('spaces.speech.language')"
        :options="languageOptions"
        :disabled="disabled"
      />
      <UiInput
        v-model="speed"
        class="min-w-0 flex-1"
        size="sm"
        type="number"
        min="0.5"
        max="2"
        step="0.05"
        :disabled="disabled || activeRevision?.engineId === 'voxcpm2'"
      />
      <UiButton
        size="sm"
        :disabled="
          disabled ||
          !isSpeedValid ||
          (language === activeRevision?.language && Number(speed) === activeRevision?.speed)
        "
        @click="emit('revise', language, Number(speed))"
        >{{ t("spaces.speech.newVoiceRevision") }}</UiButton
      >
    </div>
    <VoiceRevisionHistory :profile :disabled />
    <div v-if="targets.length" class="flex gap-2">
      <UiSelect
        v-model="targetSpaceId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.targetSpace')"
        :options="targets"
        :disabled="disabled"
      />
      <UiInput v-model="copyName" class="min-w-0 flex-1" size="sm" :disabled="disabled" />
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="disabled || !targetSpaceId || !copyName.trim()"
        @click="emit('copy', targetSpaceId, copyName.trim())"
        ><Copy class="size-3.5" />{{ t("spaces.speech.copyVoice") }}</UiButton
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
