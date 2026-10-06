<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { readSkillFile } from "../../../foundation/extensions/skill-operations";
import UiButton from "../../../shared/ui/UiButton.vue";
const props = defineProps<{ skillId: string; files: string[] }>();
const reading = useMutation({ mutationFn: (path: string) => readSkillFile(props.skillId, path) });
</script>

<template>
  <section class="space-y-2" data-testid="skill-resource-browser">
    <div class="flex flex-wrap gap-1">
      <UiButton
        v-for="path in files"
        :key="path"
        size="sm"
        variant="ghost"
        :disabled="reading.isPending.value"
        @click="reading.mutate(path)"
        >{{ path }}</UiButton
      >
    </div>
    <p v-if="reading.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(reading.error.value) }}
    </p>
    <template v-if="reading.data.value">
      <p class="text-caption break-all text-fg-tertiary">
        {{ reading.data.value.path }} · {{ reading.data.value.digest }}
      </p>
      <pre
        v-if="reading.data.value.encoding === 'utf8'"
        class="max-h-64 overflow-auto text-caption break-all whitespace-pre-wrap"
        >{{ reading.data.value.content }}</pre>
      <p v-else class="text-caption text-fg-secondary">{{ $t("extensions.binarySkillResource") }}</p>
    </template>
  </section>
</template>
