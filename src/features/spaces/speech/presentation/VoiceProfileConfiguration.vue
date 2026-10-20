<script setup lang="ts">
import { Archive, AudioLines, ChevronDown, MoreHorizontal, RotateCcw, Trash2 } from "@lucide/vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, shallowRef, watch, type DeepReadonly } from "vue";
import { useI18n } from "vue-i18n";

import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import { LABEL_BY_VOICE_ENGINE } from "../voice-options";
import { activeVoiceProfileRevision } from "../voice-profile";
import VoiceProfileEditor from "./VoiceProfileEditor.vue";

const props = defineProps<{
  profile: DeepReadonly<VoiceProfileHistory>;
  expanded: boolean;
  disabled: boolean;
}>();
const emit = defineEmits<{
  dirty: [value: boolean];
  toggle: [];
  rename: [name: string];
  revise: [language: string, speed: number];
  archive: [];
  unarchive: [];
  remove: [];
}>();
const { t } = useI18n();
const isEditorDirty = shallowRef(false);
const activeRevision = computed(() => activeVoiceProfileRevision(props.profile));
watch(
  () => props.expanded,
  (expanded) => {
    if (!expanded) isEditorDirty.value = false;
  },
);
const menuItems = computed(() => [
  {
    label: props.profile.archivedAt ? t("common.restore") : t("common.archive"),
    icon: props.profile.archivedAt ? RotateCcw : Archive,
    disabled: props.disabled,
    onSelect: () => {
      if (props.profile.archivedAt) emit("unarchive");
      else emit("archive");
    },
  },
  ...(props.profile.archivedAt
    ? [
        { separator: true },
        {
          label: t("common.delete"),
          icon: Trash2,
          destructive: true,
          disabled: props.disabled || props.profile.referencedBindingCount > 0,
          onSelect: remove,
        },
      ]
    : []),
]);

async function remove() {
  const confirmed = await confirmAction({
    title: t("spaces.speech.voice.delete.title"),
    description: t("spaces.speech.voice.delete.description"),
    confirmLabel: t("common.delete"),
    danger: true,
  });
  if (confirmed) emit("remove");
}

function toggle(open: boolean) {
  if (open !== props.expanded) emit("toggle");
}

function setDirty(value: boolean) {
  isEditorDirty.value = value;
  emit("dirty", value);
}
</script>

<template>
  <CollapsibleRoot :open="expanded" as-child @update:open="toggle">
    <article>
      <div class="flex min-h-(--knovum-setting-min-height) items-center gap-3 px-4 py-3">
        <AudioLines class="size-4 shrink-0 text-fg-secondary" :stroke-width="1.55" />
        <div class="min-w-0 flex-1">
          <div class="flex min-w-0 items-center gap-2">
            <strong class="truncate text-body-lg font-medium text-fg-primary">{{ profile.name }}</strong>
            <UiBadge v-if="profile.archivedAt" size="sm">{{ t("spaces.speech.voice.archived") }}</UiBadge>
          </div>
          <p class="mt-0.5 truncate text-caption text-fg-secondary">
            {{ t(`spaces.speech.voice.origins.${profile.sourceKind}`) }} · {{ activeRevision.language }} ·
            {{ LABEL_BY_VOICE_ENGINE[activeRevision.engineId] }} ·
            {{
              t(
                "spaces.speech.voice.usage",
                { count: profile.referencedBindingCount },
                profile.referencedBindingCount,
              )
            }}
          </p>
        </div>
        <UiDropdownMenu :items="menuItems">
          <template #trigger>
            <UiIconButton :label="t('common.more')" size="sm" :disabled="disabled || isEditorDirty">
              <MoreHorizontal class="size-3.5" />
            </UiIconButton>
          </template>
        </UiDropdownMenu>
        <CollapsibleTrigger as-child>
          <UiIconButton :label="t('common.edit')" size="sm" :active="expanded" :disabled>
            <ChevronDown
              class="size-3.5 transition-transform duration-100"
              :class="expanded && 'rotate-180'"
            />
          </UiIconButton>
        </CollapsibleTrigger>
      </div>
      <CollapsibleContent class="border-t border-stroke-divider px-4 py-4">
        <VoiceProfileEditor
          :profile
          :disabled
          @dirty="setDirty"
          @rename="emit('rename', $event)"
          @revise="(language, speed) => emit('revise', language, speed)"
        />
      </CollapsibleContent>
    </article>
  </CollapsibleRoot>
</template>
