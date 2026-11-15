<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { AudiobookRoleDraft } from "../audiobook-draft";

defineProps<{
  roles: AudiobookRoleDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  usedRoleKeys: readonly string[];
  roleUsageCounts?: ReadonlyMap<string, number> | undefined;
  disabled: boolean;
}>();
const emit = defineEmits<{
  add: [];
  rename: [role: AudiobookRoleDraft, value: string];
  voice: [role: AudiobookRoleDraft, value: string];
  remove: [role: AudiobookRoleDraft];
}>();
const { t } = useI18n();
const roleBindingsRef = useTemplateRef<HTMLElement>("roleBindingsRef");
async function addRole() {
  emit("add");
  await nextTick();
  const fields = roleBindingsRef.value?.querySelectorAll<HTMLInputElement>("input");
  const field = fields?.[fields.length - 1];
  field?.focus();
  field?.select();
}
</script>

<template>
  <section ref="roleBindingsRef" class="space-y-3">
    <div class="flex items-center justify-between gap-2">
      <div class="min-w-0 flex-1 space-y-0.5">
        <p class="text-caption font-medium text-fg-primary">
          {{ t("spaces.speech.project.roleBindings.label") }}
        </p>
        <p class="text-caption-sm text-fg-tertiary">
          {{ t("spaces.speech.project.roleBindings.hint") }}
        </p>
      </div>
      <UiButton size="sm" variant="secondary" :disabled="disabled" @click="addRole">
        <Plus class="size-3.5" />{{ t("spaces.speech.project.addRole") }}
      </UiButton>
    </div>
    <div class="divide-y divide-stroke-divider rounded-md border border-stroke-default bg-surface-raised">
      <div v-for="role in roles" :key="role.id" class="flex items-center gap-1.5 px-2 py-1.5">
        <UiInput
          :model-value="role.key"
          class="w-24 shrink-0 border-transparent bg-transparent shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus focus:bg-surface-raised"
          size="sm"
          :aria-label="t('spaces.speech.project.role')"
          :placeholder="t('spaces.speech.project.role')"
          :disabled="disabled"
          @update:model-value="emit('rename', role, $event)"
        />
        <UiSelect
          :model-value="role.voiceRevisionId"
          class="min-w-0 flex-1 border-transparent bg-transparent shadow-none hover:border-transparent hover:bg-surface-control focus:border-stroke-focus"
          size="sm"
          :label="t('spaces.speech.voice.label')"
          :placeholder="t('spaces.speech.library.selection.placeholder')"
          :options="voiceOptions"
          :disabled="disabled"
          @update:model-value="emit('voice', role, $event)"
        />
        <UiBadge v-if="roleUsageCounts?.get(role.key)" size="sm" variant="neutral">
          {{ roleUsageCounts.get(role.key) }}
        </UiBadge>
        <UiIconButton
          size="sm"
          tooltip
          class="hover:text-fg-danger"
          :label="t('spaces.speech.project.removeRole')"
          :disabled="disabled || usedRoleKeys.includes(role.key)"
          @click="emit('remove', role)"
        >
          <Trash2 class="size-3.5" />
        </UiIconButton>
      </div>
    </div>
  </section>
</template>
