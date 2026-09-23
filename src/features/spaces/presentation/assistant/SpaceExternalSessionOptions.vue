<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiSelect, { type SelectOption } from "../../../../shared/ui/UiSelect.vue";

interface SessionOption {
  id: string;
  name: string;
  description?: string;
  type: "select" | "boolean";
  currentValue: string | boolean;
  options: SelectOption[];
}

const props = defineProps<{ capabilities: unknown; disabled: boolean }>();
const emit = defineEmits<{
  change: [id: string, value: string | boolean];
  mode: [id: string];
}>();
const options = computed(() => parseOptions(props.capabilities));
const mode = computed(() => parseMode(props.capabilities));
const { t } = useI18n();

function parseOptions(capabilities: unknown): SessionOption[] {
  if (!isRecord(capabilities) || !Array.isArray(capabilities.configOptions)) return [];
  return capabilities.configOptions.flatMap<SessionOption>((value): SessionOption[] => {
    if (!isRecord(value) || typeof value.id !== "string" || typeof value.name !== "string") return [];
    if (value.type === "boolean" && typeof value.currentValue === "boolean") {
      return [{ ...baseOption(value), type: "boolean", currentValue: value.currentValue, options: [] }];
    }
    if (value.type !== "select" || typeof value.currentValue !== "string" || !Array.isArray(value.options)) {
      return [];
    }
    const choices = value.options
      .flatMap((entry) => (isRecord(entry) && Array.isArray(entry.options) ? entry.options : [entry]))
      .flatMap(parseChoice);
    return [{ ...baseOption(value), type: "select", currentValue: value.currentValue, options: choices }];
  });
}

function parseMode(capabilities: unknown): SessionOption | undefined {
  if (
    !isRecord(capabilities) ||
    !isRecord(capabilities.modes) ||
    typeof capabilities.modes.currentModeId !== "string" ||
    !Array.isArray(capabilities.modes.availableModes)
  )
    return;
  const choices = capabilities.modes.availableModes.flatMap((value) => {
    if (!isRecord(value) || typeof value.id !== "string" || typeof value.name !== "string") return [];
    return [
      {
        value: value.id,
        label: value.name,
        ...(typeof value.description === "string" ? { description: value.description } : {}),
      },
    ];
  });
  return {
    id: "mode",
    name: t("spaces.assistant.externalSessionMode"),
    type: "select",
    currentValue: capabilities.modes.currentModeId,
    options: choices,
  };
}

function baseOption(value: Record<string, unknown>) {
  return {
    id: value.id as string,
    name: value.name as string,
    ...(typeof value.description === "string" ? { description: value.description } : {}),
  };
}

function parseChoice(value: unknown): SelectOption[] {
  if (!isRecord(value) || typeof value.value !== "string" || typeof value.name !== "string") return [];
  return [
    {
      value: value.value,
      label: value.name,
      ...(typeof value.description === "string" ? { description: value.description } : {}),
    },
  ];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
</script>

<template>
  <div v-if="mode || options.length" class="space-y-2 rounded-panel border border-border p-2">
    <UiSelect
      v-if="mode"
      :model-value="String(mode.currentValue)"
      :label="mode.name"
      :options="mode.options"
      :disabled="disabled"
      @update:model-value="emit('mode', $event)"
    />
    <div v-for="option in options" :key="option.id">
      <UiSelect
        v-if="option.type === 'select'"
        :model-value="String(option.currentValue)"
        :label="option.name"
        :options="option.options"
        :disabled="disabled"
        @update:model-value="emit('change', option.id, $event)"
      />
      <label v-else class="flex items-start gap-2 text-label text-foreground">
        <UiCheckbox
          :model-value="Boolean(option.currentValue)"
          :label="option.name"
          :disabled="disabled"
          @update:model-value="emit('change', option.id, $event)"
        />
        <span>
          <span class="block">{{ option.name }}</span>
          <span v-if="option.description" class="block text-caption text-foreground-tertiary">
            {{ option.description }}
          </span>
        </span>
      </label>
    </div>
  </div>
</template>
