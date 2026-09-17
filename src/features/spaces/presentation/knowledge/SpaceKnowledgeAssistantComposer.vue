<script setup lang="ts">
import { ArrowUp, CircleStop } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { GroundedScope } from "../../knowledge/model/grounded-scope";
import SpaceKnowledgeScope from "./SpaceKnowledgeScope.vue";

const props = defineProps<{
  busy: boolean;
  scopeLocked: boolean;
  selectionCount: number;
  sectionCount: number;
  spaceCount: number;
  scopeTitles: readonly string[];
}>();
const emit = defineEmits<{ ask: []; cancel: [] }>();
const question = defineModel<string>({ required: true });
const executionTarget = defineModel<ExecutionTarget | undefined>("executionTarget");
const scope = defineModel<GroundedScope>("scope", { required: true });
const { t } = useI18n();
const canSubmit = computed(
  () => !props.busy && Boolean(question.value.trim()) && Boolean(executionTarget.value),
);

function submit() {
  if (canSubmit.value) emit("ask");
}
</script>

<template>
  <footer class="shrink-0 border-t border-divider bg-panel p-2.5">
    <div
      class="rounded-panel border border-border bg-raised transition-colors focus-within:border-focus-ring"
    >
      <UiTextarea
        v-model="question"
        :rows="3"
        class="min-h-20 resize-none rounded-b-none border-0 bg-transparent px-3 py-2.5 hover:border-transparent focus:border-transparent"
        :placeholder="t('spaces.assistant.askSpacePlaceholder')"
        @keydown.meta.enter.prevent="submit"
        @keydown.ctrl.enter.prevent="submit"
      />
      <div class="flex min-w-0 items-center gap-1.5 px-2 pb-2">
        <SpaceKnowledgeScope
          v-model="scope"
          class="min-w-0 flex-1"
          compact
          :locked="scopeLocked"
          :selection-count="selectionCount"
          :section-count="sectionCount"
          :space-count="spaceCount"
          :titles="scopeTitles"
        />
        <AiExecutionTargetField
          v-model="executionTarget"
          class="w-32 shrink-0"
          purpose="knowledge"
          compact
          :disabled="busy"
        />
        <UiIconButton
          v-if="busy"
          :label="t('spaces.assistant.stop')"
          size="sm"
          class="rounded-full bg-control-pressed text-foreground"
          @click="emit('cancel')"
        >
          <CircleStop class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <UiIconButton
          v-else
          :label="t('spaces.assistant.ask')"
          shortcut="$mod+Enter"
          size="sm"
          class="rounded-full bg-accent text-on-accent hover:bg-accent-hover hover:text-on-accent"
          :disabled="!canSubmit"
          @click="submit"
        >
          <ArrowUp class="size-(--knovum-icon-size-sm)" :stroke-width="1.8" />
        </UiIconButton>
      </div>
    </div>
  </footer>
</template>
