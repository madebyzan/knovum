<script setup lang="ts">
import { Plus, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import type { AudiobookRoleDraft } from "./use-audiobook-draft";

defineProps<{
  roles: AudiobookRoleDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  usedRoleKeys: readonly string[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  add: [];
  rename: [role: AudiobookRoleDraft, value: string];
  remove: [role: AudiobookRoleDraft];
}>();
const { t } = useI18n();
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-divider p-2">
    <div class="flex items-center justify-between gap-2">
      <div>
        <p class="text-caption font-medium text-fg-primary">{{ t("spaces.speech.roleBindings") }}</p>
        <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.speech.roleBindingsHint") }}</p>
      </div>
      <UiButton size="sm" variant="ghost" :disabled="disabled" @click="emit('add')">
        <Plus class="size-3.5" />{{ t("spaces.speech.addRole") }}
      </UiButton>
    </div>
    <div v-for="role in roles" :key="role.id" class="flex items-center gap-1">
      <UiInput
        :model-value="role.key"
        class="min-w-0 flex-1"
        size="sm"
        :placeholder="t('spaces.speech.role')"
        :disabled="disabled"
        @update:model-value="emit('rename', role, $event)"
      />
      <UiSelect
        v-model="role.voiceRevisionId"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voice')"
        :options="voiceOptions"
        :disabled="disabled"
      />
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :aria-label="t('spaces.speech.removeRole')"
        :disabled="disabled || usedRoleKeys.includes(role.key)"
        @click="emit('remove', role)"
      >
        <Trash2 class="size-3.5" />
      </UiButton>
    </div>
  </section>
</template>
