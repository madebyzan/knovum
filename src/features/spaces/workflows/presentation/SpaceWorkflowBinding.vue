<script setup lang="ts">
import { ArrowDown, ArrowUp } from "@lucide/vue";
import { compact, without } from "es-toolkit";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect, { type SelectOption } from "../../../../shared/ui/UiSelect.vue";
import type { CaptureCandidate } from "../../contracts/generated/CaptureCandidate";

const props = defineProps<{
  slot: CaptureCandidate["inputSlots"][number];
  label: string;
  options: SelectOption[];
  modelValue: string[];
}>();
const emit = defineEmits<{ "update:modelValue": [value: string[]] }>();
const { t } = useI18n();
const draggingNodeId = ref("");
const selected = computed(() => new Set(props.modelValue));
const selectedOptions = computed(() =>
  compact(props.modelValue.map((id) => props.options.find((option) => option.value === id))),
);

function toggle(nodeId: string, enabled: boolean) {
  const next = enabled ? [...props.modelValue, nodeId] : without(props.modelValue, nodeId);
  emit("update:modelValue", next);
}

function move(index: number, offset: number) {
  const next = [...props.modelValue];
  const target = index + offset;
  if (target < 0 || target >= next.length) return;
  [next[index], next[target]] = [next[target]!, next[index]!];
  emit("update:modelValue", next);
}

function dropOn(targetNodeId: string) {
  const source = props.modelValue.indexOf(draggingNodeId.value);
  const target = props.modelValue.indexOf(targetNodeId);
  draggingNodeId.value = "";
  if (source < 0 || target < 0 || source === target) return;
  const next = [...props.modelValue];
  next.splice(target, 0, next.splice(source, 1)[0]!);
  emit("update:modelValue", next);
}

function beginDrag(event: DragEvent, nodeId: string) {
  draggingNodeId.value = nodeId;
  event.dataTransfer?.setData("text/plain", nodeId);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}
</script>

<template>
  <div class="space-y-2 rounded-md border border-stroke-default p-2.5">
    <p class="text-caption font-medium text-fg-secondary">{{ label }} · {{ slot.kind }}</p>
    <UiSelect
      v-if="!slot.ordered"
      :model-value="modelValue[0] ?? ''"
      :label="label"
      :options="options"
      :placeholder="t('spaces.creation.workflow.inputs.select')"
      @update:model-value="emit('update:modelValue', [$event])"
    />
    <template v-else>
      <p class="text-caption-sm text-fg-tertiary">
        {{ t("spaces.creation.workflow.inputs.sequenceLimit", { count: slot.maxItems }) }}
      </p>
      <div class="space-y-1">
        <label
          v-for="option in options"
          :key="option.value"
          class="flex cursor-pointer items-center gap-2 rounded-md px-1 py-1 hover:bg-surface-control-hover"
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
      <ol v-if="selectedOptions.length" class="space-y-1 border-t border-stroke-divider pt-2">
        <li
          v-for="(option, index) in selectedOptions"
          :key="option.value"
          draggable="true"
          class="flex items-center gap-1 rounded-md"
          :class="draggingNodeId === option.value ? 'opacity-50' : ''"
          @dragstart="beginDrag($event, option.value)"
          @dragover.prevent
          @drop.prevent="dropOn(option.value)"
          @dragend="draggingNodeId = ''"
        >
          <span class="w-5 text-right text-caption-sm text-fg-tertiary tabular-nums">{{ index + 1 }}</span>
          <span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="index === 0"
            :aria-label="t('spaces.creation.workflow.inputs.moveUp')"
            @click="move(index, -1)"
          >
            <ArrowUp class="size-3" />
          </UiButton>
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="index === selectedOptions.length - 1"
            :aria-label="t('spaces.creation.workflow.inputs.moveDown')"
            @click="move(index, 1)"
          >
            <ArrowDown class="size-3" />
          </UiButton>
        </li>
      </ol>
    </template>
  </div>
</template>
