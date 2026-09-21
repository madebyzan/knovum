<script setup lang="ts">
import { Check, ChevronDown, LibraryBig, LockKeyhole } from "@lucide/vue";
import { take } from "es-toolkit";
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuTrigger,
} from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { GroundedScope } from "../../knowledge/model/grounded-scope";

interface ScopeOption {
  value: GroundedScope;
  count?: number;
}

const modelValue = defineModel<GroundedScope>({ required: true });
const props = defineProps<{
  locked?: boolean;
  selectionCount: number;
  sectionCount: number;
  spaceCount: number;
  titles?: readonly string[];
  compact?: boolean;
}>();
const { t } = useI18n();
const scopeOptions = computed<ScopeOption[]>(() => [
  { value: "selection" as const, count: props.selectionCount },
  { value: "section" as const, count: props.sectionCount },
  { value: "space" as const, count: props.spaceCount },
  { value: "all-spaces" as const },
]);
const activeScope = computed(
  () => scopeOptions.value.find((option) => option.value === modelValue.value) ?? scopeOptions.value[2]!,
);
function scopeLabel(option: ScopeOption) {
  const label = t(`spaces.assistant.aiScope.${option.value}`);
  return option.count === undefined ? label : `${label} · ${option.count}`;
}
</script>

<template>
  <div>
    <div class="flex min-w-0 items-center">
      <div
        v-if="locked"
        class="flex h-(--knovum-control-height-sm) min-w-0 flex-1 items-center gap-1.5 rounded-control px-2 text-caption text-foreground-secondary"
        :class="compact ? 'bg-transparent' : 'bg-control/35'"
      >
        <LibraryBig class="size-(--knovum-icon-size-xs) shrink-0" :stroke-width="1.55" />
        <span class="min-w-0 flex-1 truncate">{{ scopeLabel(activeScope) }}</span>
        <LockKeyhole
          class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
          :stroke-width="1.55"
        />
      </div>
      <DropdownMenuRoot v-else>
        <DropdownMenuTrigger as-child>
          <button
            type="button"
            class="flex h-(--knovum-control-height-sm) min-w-0 flex-1 items-center gap-1.5 rounded-control px-2 text-caption text-foreground-secondary outline-none hover:bg-control-hover hover:text-foreground"
            :class="compact ? 'bg-transparent' : 'bg-control/30'"
          >
            <LibraryBig class="size-(--knovum-icon-size-xs) shrink-0" :stroke-width="1.55" />
            <span class="min-w-0 flex-1 truncate text-left">{{ scopeLabel(activeScope) }}</span>
            <ChevronDown
              class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
              :stroke-width="1.55"
            />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuPortal>
          <DropdownMenuContent
            align="start"
            :side-offset="6"
            class="z-50 min-w-52 rounded-panel bg-overlay p-1 shadow-popover ring-1 ring-border"
          >
            <DropdownMenuItem
              v-for="option in scopeOptions"
              :key="option.value"
              class="flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label outline-none data-disabled:opacity-35 data-highlighted:bg-control-hover"
              :disabled="(option.value === 'selection' || option.value === 'section') && option.count === 0"
              @select="modelValue = option.value"
            >
              <Check
                class="size-(--knovum-icon-size-xs)"
                :class="modelValue === option.value ? 'opacity-100' : 'opacity-0'"
              />
              <span class="min-w-0 flex-1">{{ t(`spaces.assistant.aiScope.${option.value}`) }}</span>
              <span
                v-if="option.count !== undefined"
                class="text-micro text-foreground-tertiary tabular-nums"
                >{{ option.count }}</span
              >
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenuPortal>
      </DropdownMenuRoot>
    </div>
    <div
      v-if="!compact && modelValue === 'selection' && titles?.length"
      class="mt-1.5 flex min-w-0 flex-wrap items-center gap-1"
    >
      <span
        v-for="(title, index) in take(titles, 3)"
        :key="`${index}:${title}`"
        class="max-w-32 shrink truncate rounded-control bg-control/35 px-1.5 py-0.5 text-micro text-foreground-secondary"
      >
        {{ title }}
      </span>
      <span v-if="titles.length > 3" class="shrink-0 text-micro text-foreground-tertiary"
        >+{{ titles.length - 3 }}</span
      >
    </div>
  </div>
</template>
