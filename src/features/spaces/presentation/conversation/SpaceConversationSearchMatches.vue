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
const parsed = computed(() => searchResponseSchema.safeParse(props.value));
const matches = computed(() => {
  const response = parsed.value;
  return response.success ? response.data.result : [];
});
</script>

<template>
  <section class="mt-2 text-caption">
    <p class="text-fg-tertiary">
      {{ t("spaces.assistant.sources", { count: matches.length }) }}
    </p>
    <p v-if="!parsed.success" class="mt-1 text-fg-danger" role="alert">
      {{ t("spaces.assistant.activity.invalidResult") }}
    </p>
    <p v-else-if="!matches.length" class="mt-1 text-fg-tertiary">
      {{ t("spaces.assistant.activity.noSources") }}
    </p>
    <ul class="mt-2 space-y-1.5">
      <li
        v-for="match in matches"
        :key="`${match.nodeId}:${match.sourceRevision}`"
        class="rounded-md bg-surface-control p-2"
      >
        <button
          type="button"
          class="text-left font-medium text-fg-accent hover:underline"
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
        <p class="mt-1 line-clamp-3 text-fg-secondary">{{ match.preview }}</p>
      </li>
    </ul>
  </section>
</template>
