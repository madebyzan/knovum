<script setup lang="ts">
import { LoaderCircle, TriangleAlert, X } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiTooltip from "../../../../../shared/ui/UiTooltip.vue";
import { useCanvasGeneratorStore } from "../generator-store";
import type { ReferenceView } from "./use-reference-readings";

/**
 * What a generator makes from, above its prompt: a picture as a thumbnail, anything else by name.
 * Each says on itself whether it is still being prepared or cannot be read, so a blocked Generate
 * button is never a mystery.
 */
defineProps<{ generatorId: string; references: readonly ReferenceView[] }>();
const { t } = useI18n();
const store = useCanvasGeneratorStore();

function status(reference: ReferenceView) {
  if (reference.state === "preparing")
    return t("spaces.creation.generator.references.preparing", { title: reference.title });
  if (reference.state === "unreadable")
    return t("spaces.creation.generator.references.unreadable", { title: reference.title });
  return reference.title;
}
</script>

<template>
  <ul class="flex flex-wrap gap-2 px-3 pt-3">
    <li v-for="reference in references" :key="reference.nodeId" class="group relative">
      <UiTooltip :text="status(reference)" side="top">
        <div
          class="flex h-11 items-center overflow-hidden rounded-md border bg-surface-control"
          :class="[
            reference.state === 'unreadable' ? 'border-stroke-warning' : 'border-stroke-default',
            reference.preview ? 'w-11' : 'max-w-44 gap-2 pr-3 pl-1.5',
          ]"
          tabindex="0"
          :aria-label="status(reference)"
        >
          <img
            v-if="reference.preview"
            :src="reference.preview"
            alt=""
            class="size-full object-cover"
            :class="{ 'opacity-50': reference.state !== 'ready' }"
          />
          <template v-else>
            <span
              class="grid size-8 shrink-0 place-items-center rounded-sm bg-surface-raised text-fg-secondary"
            >
              <component :is="reference.icon" class="size-4" :stroke-width="1.6" />
            </span>
            <span class="min-w-0 truncate text-caption text-fg-primary">{{ reference.title }}</span>
          </template>
          <span
            v-if="reference.state !== 'ready'"
            class="pointer-events-none absolute grid place-items-center"
            :class="reference.preview ? 'inset-0' : 'top-1.5 left-1.5 size-8 rounded-sm bg-surface-raised'"
          >
            <LoaderCircle
              v-if="reference.state === 'preparing'"
              class="size-4 animate-spin text-fg-secondary"
              :stroke-width="2"
            />
            <TriangleAlert v-else class="size-4 text-fg-warning" :stroke-width="2" />
          </span>
        </div>
      </UiTooltip>
      <button
        type="button"
        class="absolute -top-1.5 -right-1.5 grid size-5 place-items-center rounded-full border border-stroke-default bg-surface-raised text-fg-secondary opacity-0 shadow-control transition-opacity group-focus-within:opacity-100 group-hover:opacity-100 hover:text-fg-primary focus-visible:opacity-100"
        :aria-label="t('spaces.creation.generator.references.remove', { title: reference.title })"
        @click="store.removeReference(generatorId, reference.nodeId)"
      >
        <X class="size-3" :stroke-width="2" />
      </button>
    </li>
  </ul>
</template>
