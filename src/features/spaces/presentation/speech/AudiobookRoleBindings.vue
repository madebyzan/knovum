<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { AudiobookRoleDraft } from "../../speech/audiobook-draft";

defineProps<{
  roles: AudiobookRoleDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  usedRoleKeys: readonly string[];
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
      <div class="min-w-0 flex-1">
        <p class="text-caption font-medium text-fg-primary">{{ t("spaces.speech.project.roleBindings") }}</p>
      </div>
      <UiButton size="sm" variant="ghost" :disabled="disabled" @click="addRole">
        <Plus class="size-3.5" />{{ t("spaces.speech.project.addRole") }}
      </UiButton>
    </div>
    <p class="text-caption-sm leading-relaxed text-fg-tertiary">
      {{ t("spaces.speech.project.roleBindingsHint") }}
    </p>
    <div v-for="role in roles" :key="role.id" class="flex items-center gap-1">
      <UiInput
        :model-value="role.key"
        class="w-24 min-w-0"
        size="sm"
        :aria-label="t('spaces.speech.project.role')"
        :placeholder="t('spaces.speech.project.role')"
        :disabled="disabled"
        @update:model-value="emit('rename', role, $event)"
      />
      <UiSelect
        :model-value="role.voiceRevisionId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voice.label')"
        :options="voiceOptions"
        :disabled="disabled"
        @update:model-value="emit('voice', role, $event)"
      />
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
  </section>
</template>
