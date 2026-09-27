<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { z } from "zod";

import { knowledgeLocatorSchema } from "../../knowledge/model/schema/chunk";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ value: unknown }>();
const store = useSpaceStore();
const { t } = useI18n();
const searchResponseSchema = z
  .object({
    type: z.literal("query_result"),
    result: z.array(
      z
        .object({
          nodeId: z.string().min(1),
          title: z.string().nullable(),
          preview: z.string(),
          sourceRevision: z.string().min(1),
          locator: knowledgeLocatorSchema,
        })
        .passthrough(),
    ),
  })
  .passthrough();
const matches = computed(() => {
  const parsed = searchResponseSchema.safeParse(props.value);
  return parsed.success ? parsed.data.result : [];
});
</script>

<template>
  <details class="rounded-control border border-divider px-2 py-1.5 text-caption">
    <summary class="cursor-pointer text-foreground-secondary select-none">
      {{ t("spaces.assistant.sources", { count: matches.length }) }}
    </summary>
    <ul class="mt-2 space-y-1.5">
      <li
        v-for="match in matches"
        :key="`${match.nodeId}:${match.sourceRevision}`"
        class="rounded-control bg-control p-2"
      >
        <button
          type="button"
          class="text-left font-medium text-accent hover:underline"
          @click="
            store.openSource({
              nodeId: match.nodeId,
              nodeTitle: match.title ?? match.nodeId,
              locator: match.locator,
              text: match.preview,
            })
          "
        >
          {{ match.title ?? match.nodeId }}
        </button>
        <p class="mt-1 line-clamp-3 text-foreground-secondary">{{ match.preview }}</p>
      </li>
    </ul>
  </details>
</template>
