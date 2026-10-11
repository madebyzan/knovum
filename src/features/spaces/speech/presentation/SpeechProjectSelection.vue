<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpeechProjectOverview } from "../../contracts/generated/SpeechProjectOverview";

defineProps<{ projects: readonly SpeechProjectOverview[]; selectedId: string; disabled: boolean }>();
const emit = defineEmits<{ select: [id: string] }>();
const { d, t } = useI18n();
</script>

<template>
  <div class="flex shrink-0 items-center gap-2 px-3 pt-2">
    <UiSelect
      v-if="projects.length > 1"
      size="sm"
      class="min-w-0 flex-1"
      :model-value="selectedId"
      :options="
        projects.map((project) => ({
          value: project.id,
          label: `${project.title} · ${d(project.updatedAt, 'short')}`,
        }))
      "
      :label="t('spaces.speech.project.nodeProjects')"
      :disabled
      @update:model-value="emit('select', $event)"
    />
  </div>
</template>
