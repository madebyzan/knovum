<script setup lang="ts">
import { ArrowUpRight, Plus, X } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { spaceCanvasOverviewSource } from "../../assets/node-preview-source";
import { nodeIcon } from "../../canvas/presentation/node-icon";
import { useSpaceStore } from "../../session/space-store";
import { knowledgeLocationLabel } from "../model/location-label";
import type { SpaceProvenanceTargetKind } from "../model/provenance";
import { attachProvenanceFromPeek, detachProvenance } from "../provenance-actions";
import { useSpaceProvenanceQuery } from "../queries/use-space-provenance-query";

const props = defineProps<{
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
  attachNodeIds?: readonly string[];
  removable?: boolean;
}>();
const store = useSpaceStore();
const { activeSpaceId, nodes, readingTarget } = storeToRefs(store);
const { t } = useI18n();
const provenanceQuery = useSpaceProvenanceQuery(activeSpaceId, props.targetKind, () => props.targetId);
const provenance = computed(() => provenanceQuery.data.value ?? []);
const mutating = shallowRef(false);
const nodeById = computed(() => new Map(nodes.value.map((node) => [node.id, node])));
const attachable = computed(() => {
  const reading = readingTarget.value;
  if (!reading?.locator || reading.nodeId === props.targetId) return undefined;
  if (props.attachNodeIds && !props.attachNodeIds.includes(reading.nodeId)) return undefined;
  const node = nodeById.value.get(reading.nodeId);
  return {
    nodeId: reading.nodeId,
    nodeTitle: node?.data.title ?? reading.nodeId,
    locator: reading.locator,
    ...(reading.quote ? { text: reading.quote } : {}),
  };
});
const sources = computed(() =>
  provenance.value.map((record) => {
    const node = nodeById.value.get(record.sourceNodeId);
    return {
      record,
      title: node?.data.title ?? record.sourceNodeId,
      kind: node?.data.kind ?? "file",
      thumbnail: node ? spaceCanvasOverviewSource(node.data) : "",
    };
  }),
);

async function mutate(action: () => Promise<unknown>) {
  if (mutating.value) return;
  mutating.value = true;
  try {
    await action();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    mutating.value = false;
  }
}
async function attachCurrent() {
  const peek = attachable.value;
  const spaceId = activeSpaceId.value;
  if (!peek || !spaceId) return;
  await mutate(() =>
    attachProvenanceFromPeek(
      {
        spaceId,
        targetKind: props.targetKind,
        targetId: props.targetId,
      },
      peek,
    ),
  );
}
async function remove(provenanceId: string) {
  const spaceId = activeSpaceId.value;
  if (!spaceId) return;
  await mutate(() =>
    detachProvenance(provenanceId, {
      spaceId,
      targetKind: props.targetKind,
      targetId: props.targetId,
    }),
  );
}
function openSource(source: (typeof sources.value)[number]) {
  const { record } = source;
  store.openSourceReader({
    nodeId: record.sourceNodeId,
    locator: record.locator,
    quote: record.quote,
    pane: "read",
  });
}
</script>

<template>
  <section v-if="sources.length || attachable">
    <div class="flex items-center gap-2">
      <h5 class="min-w-0 flex-1 text-caption font-medium text-fg-tertiary">
        {{ t("spaces.knowledge.provenance") }}
      </h5>
      <UiIconButton
        v-if="attachable"
        :label="t('spaces.conversation.evidence.useCurrent')"
        tooltip
        size="sm"
        :disabled="mutating"
        @click="attachCurrent"
      >
        <Plus class="size-3" />
      </UiIconButton>
    </div>
    <div v-if="sources.length" class="mt-1 space-y-0.5">
      <div v-for="source in sources" :key="source.record.id" class="group flex items-start gap-1">
        <button
          type="button"
          class="-mx-1.5 flex min-w-0 flex-1 items-start gap-2.5 rounded-md px-1.5 py-1.5 text-left transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
          @click="openSource(source)"
        >
          <span
            class="grid size-7 shrink-0 place-items-center overflow-hidden rounded-[5px] bg-surface-control text-fg-tertiary ring-1 ring-stroke-divider/60 ring-inset"
          >
            <img
              v-if="source.thumbnail"
              :src="source.thumbnail"
              alt=""
              class="size-full object-cover"
              loading="lazy"
              decoding="async"
            />
            <component :is="nodeIcon(source.kind)" v-else class="size-3.5" :stroke-width="1.5" />
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-caption font-medium text-fg-primary">{{ source.title }}</span>
            <span class="block truncate text-caption-sm text-fg-tertiary">
              {{ knowledgeLocationLabel(source.record.locator) }}
            </span>
            <span v-if="source.record.quote" class="mt-0.5 line-clamp-2 block text-caption text-fg-secondary">
              “{{ source.record.quote }}”
            </span>
          </span>
          <ArrowUpRight
            class="mt-1 size-3 shrink-0 text-fg-tertiary opacity-0 transition-opacity duration-100 group-focus-within:opacity-100 group-hover:opacity-100"
            :stroke-width="1.6"
          />
        </button>
        <UiIconButton
          v-if="removable && source.record.status === 'active'"
          :label="t('spaces.conversation.evidence.remove')"
          tooltip
          size="sm"
          class="opacity-0 transition-opacity duration-100 group-hover:opacity-100 focus-visible:opacity-100"
          :disabled="mutating"
          @click="remove(source.record.id)"
        >
          <X class="size-3" />
        </UiIconButton>
      </div>
    </div>
  </section>
</template>
