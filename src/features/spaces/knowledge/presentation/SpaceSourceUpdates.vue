<script setup lang="ts">
import { ArrowUpRight, Check, FileClock } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useContextDock } from "../../canvas/use-context-dock";
import SpaceInspectorSection from "../../inspection/presentation/SpaceInspectorSection.vue";
import { useSpaceStore } from "../../session/space-store";
import { changedSources, currentSourceRevision, type SpaceProvenance } from "../model/provenance";
import { reviewProvenance } from "../provenance-actions";
import { useSpaceProvenanceQuery } from "../queries/use-space-provenance-query";

const props = defineProps<{ nodeId: string }>();
const store = useSpaceStore();
const dock = useContextDock();
const { activeSpaceId, nodes } = storeToRefs(store);
const { t } = useI18n();
const provenance = useSpaceProvenanceQuery(activeSpaceId, "node", () => props.nodeId);
const reviewing = shallowRef<string>();
const nodeById = computed(() => new Map(nodes.value.map((node) => [node.id, node.data])));
/** The sources this item was kept from that have changed since; the person decides what follows. */
const updates = computed(() =>
  changedSources(
    (provenance.data.value ?? []).filter(({ status }) => status === "active"),
    nodeById.value,
  ).map((record) => ({
    record,
    title: nodeById.value.get(record.sourceNodeId)?.title ?? record.sourceNodeId,
  })),
);

function open({ record, title }: (typeof updates.value)[number]) {
  dock.openSource(
    { nodeId: record.sourceNodeId, nodeTitle: title, locator: record.locator, text: record.quote },
    { reveal: false },
  );
}

async function markReviewed(record: SpaceProvenance) {
  const source = nodeById.value.get(record.sourceNodeId);
  const revision = source && currentSourceRevision(source);
  if (!revision || reviewing.value) return;
  reviewing.value = record.id;
  try {
    await reviewProvenance(record, revision);
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    reviewing.value = undefined;
  }
}
</script>

<template>
  <SpaceInspectorSection
    v-if="updates.length"
    :title="t('spaces.knowledge.sourceUpdates.title')"
    :icon="FileClock"
  >
    <p class="px-1 text-caption text-fg-tertiary">{{ t("spaces.knowledge.sourceUpdates.hint") }}</p>
    <div class="mt-1 space-y-0.5">
      <div v-for="update in updates" :key="update.record.id" class="flex items-start gap-1">
        <button
          type="button"
          class="flex min-w-0 flex-1 items-start gap-2 rounded-md px-1 py-1.5 text-left hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :title="t('spaces.knowledge.sourceUpdates.open')"
          @click="open(update)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-label text-fg-primary">{{ update.title }}</span>
            <span v-if="update.record.quote" class="mt-0.5 line-clamp-2 block text-caption text-fg-secondary">
              “{{ update.record.quote }}”
            </span>
          </span>
          <ArrowUpRight class="mt-1 size-3 shrink-0 text-fg-tertiary" :stroke-width="1.5" />
        </button>
        <UiIconButton
          :label="t('spaces.knowledge.sourceUpdates.reviewed')"
          tooltip
          size="sm"
          :disabled="Boolean(reviewing)"
          @click="markReviewed(update.record)"
        >
          <Check class="size-3" />
        </UiIconButton>
      </div>
    </div>
  </SpaceInspectorSection>
</template>
