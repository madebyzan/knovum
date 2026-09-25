<script setup lang="ts">
import { ArrowDown, ArrowUp } from "@lucide/vue";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect, { type SelectOption } from "../../../../shared/ui/UiSelect.vue";
import type { WorkflowCaptureCandidate } from "../../workflows/captured-workflow-ipc";

const props = defineProps<{
  slot: WorkflowCaptureCandidate["inputSlots"][number];
  label: string;
  options: SelectOption[];
  modelValue: string[];
}>();
const emit = defineEmits<{ "update:modelValue": [value: string[]] }>();
const { t } = useI18n();
const draggingId = ref("");
const selected = computed(() => new Set(props.modelValue));
const selectedOptions = computed(() =>
  props.modelValue
    .map((id) => props.options.find((option) => option.value === id))
    .filter((option) => option !== undefined),
);

function toggle(id: string, enabled: boolean) {
  const next = enabled ? [...props.modelValue, id] : props.modelValue.filter((candidate) => candidate !== id);
  emit("update:modelValue", next);
}

function move(index: number, offset: number) {
  const next = [...props.modelValue];
  const target = index + offset;
  if (target < 0 || target >= next.length) return;
  [next[index], next[target]] = [next[target]!, next[index]!];
  emit("update:modelValue", next);
}

function dropOn(targetId: string) {
  const source = props.modelValue.indexOf(draggingId.value);
  const target = props.modelValue.indexOf(targetId);
  draggingId.value = "";
  if (source < 0 || target < 0 || source === target) return;
  const next = [...props.modelValue];
  next.splice(target, 0, next.splice(source, 1)[0]!);
  emit("update:modelValue", next);
}

function beginDrag(event: DragEvent, id: string) {
  draggingId.value = id;
  event.dataTransfer?.setData("text/plain", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}
</script>

<template>
  <div class="space-y-2 rounded-control border border-border p-2.5">
    <p class="text-caption font-medium text-foreground-secondary">{{ label }} · {{ slot.kind }}</p>
    <UiSelect
      v-if="!slot.ordered"
      :model-value="modelValue[0] ?? ''"
      :label="label"
      :options="options"
      :placeholder="t('spaces.creation.selectWorkflowInput')"
      @update:model-value="emit('update:modelValue', [$event])"
    />
    <template v-else>
      <p class="text-micro text-foreground-tertiary">
        {{ t("spaces.creation.workflowSequenceLimit", { count: slot.maxItems }) }}
      </p>
      <div class="space-y-1">
        <label
          v-for="option in options"
          :key="option.value"
          class="flex cursor-pointer items-center gap-2 rounded-control px-1 py-1 hover:bg-control-hover"
        >
          <UiCheckbox
            :label="option.label"
            :model-value="selected.has(option.value)"
            :disabled="!selected.has(option.value) && modelValue.length >= slot.maxItems"
            @update:model-value="toggle(option.value, $event)"
          />
          <span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
        </label>
      </div>
      <ol v-if="selectedOptions.length" class="space-y-1 border-t border-divider pt-2">
        <li
          v-for="(option, index) in selectedOptions"
          :key="option.value"
          draggable="true"
          class="flex items-center gap-1 rounded-control"
          :class="draggingId === option.value ? 'opacity-50' : ''"
          @dragstart="beginDrag($event, option.value)"
          @dragover.prevent
          @drop.prevent="dropOn(option.value)"
          @dragend="draggingId = ''"
        >
          <span class="w-5 text-right text-micro text-foreground-tertiary tabular-nums">{{ index + 1 }}</span>
          <span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="index === 0"
            :aria-label="t('spaces.creation.moveWorkflowInputUp')"
            @click="move(index, -1)"
          >
            <ArrowUp class="size-(--knovum-icon-size-xs)" />
          </UiButton>
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="index === selectedOptions.length - 1"
            :aria-label="t('spaces.creation.moveWorkflowInputDown')"
            @click="move(index, 1)"
          >
            <ArrowDown class="size-(--knovum-icon-size-xs)" />
          </UiButton>
        </li>
      </ol>
    </template>
  </div>
</template>
