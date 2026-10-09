<script setup lang="ts">
import { ArrowUpRight, Plus, X } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { spaceCanvasOverviewSource } from "../../assets/node-preview-source";
import { useContextDock } from "../../canvas/use-context-dock";
import type { SpaceProvenanceTargetKind } from "../../knowledge/model/provenance";
import { attachProvenanceFromPeek, detachProvenance } from "../../knowledge/provenance-actions";
import { useSpaceProvenanceQuery } from "../../knowledge/queries/use-knowledge-queries";
import { useSpaceStore } from "../../session/space-store";
import { nodeIcon } from "../canvas/node-icon";
import { knowledgeLocationLabel } from "./knowledge-location-label";

const props = defineProps<{
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
  attachNodeIds?: readonly string[];
  removable?: boolean;
}>();
const store = useSpaceStore();
const dock = useContextDock();
const { activeSpaceId, nodes, sourcePeek } = storeToRefs(store);
const { t } = useI18n();
const provenanceQuery = useSpaceProvenanceQuery(activeSpaceId, props.targetKind, () => props.targetId);
const provenance = computed(() => provenanceQuery.data.value ?? []);
const mutating = shallowRef(false);
const attachable = computed(() => {
  const peek = sourcePeek.value;
  return peek && (!props.attachNodeIds || props.attachNodeIds.includes(peek.nodeId)) ? peek : undefined;
});

const nodeById = computed(() => new Map(nodes.value.map((node) => [node.id, node])));
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
async function remove(id: string) {
  const spaceId = activeSpaceId.value;
  if (!spaceId) return;
  await mutate(() =>
    detachProvenance(id, {
      spaceId,
      targetKind: props.targetKind,
      targetId: props.targetId,
    }),
  );
}
function openSource(source: (typeof sources.value)[number]) {
  const { record } = source;
  dock.openSource(
    {
      nodeId: record.sourceNodeId,
      nodeTitle: source.title,
      locator: record.locator,
      text: record.quote,
    },
    { reveal: false },
  );
}
</script>

<template>
  <section v-if="sources.length || attachable">
    <div class="flex items-center gap-2">
      <h5 class="min-w-0 flex-1 text-caption font-medium text-fg-secondary">
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
      <div v-for="source in sources" :key="source.record.id" class="flex items-start gap-1">
        <button
          type="button"
          class="flex min-w-0 flex-1 items-start gap-2 rounded-md px-1 py-1.5 text-left hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
          @click="openSource(source)"
        >
          <span
            class="grid size-8 shrink-0 place-items-center overflow-hidden rounded-md bg-surface-control text-fg-tertiary"
          >
            <img
              v-if="source.thumbnail"
              :src="source.thumbnail"
              alt=""
              class="size-full object-contain"
              loading="lazy"
              decoding="async"
            />
            <component :is="nodeIcon(source.kind)" v-else class="size-4" :stroke-width="1.5" />
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-label text-fg-primary">{{ source.title }}</span>
            <span class="block truncate text-caption text-fg-tertiary">
              {{ knowledgeLocationLabel(source.record.locator) }}
            </span>
            <span v-if="source.record.quote" class="mt-0.5 line-clamp-2 block text-caption text-fg-secondary">
              “{{ source.record.quote }}”
            </span>
          </span>
          <ArrowUpRight class="mt-1 size-3 shrink-0 text-fg-tertiary" :stroke-width="1.5" />
        </button>
        <UiIconButton
          v-if="removable && source.record.status === 'active'"
          :label="t('spaces.conversation.evidence.remove')"
          size="sm"
          :disabled="mutating"
          @click="remove(source.record.id)"
        >
          <X class="size-3" />
        </UiIconButton>
      </div>
    </div>
  </section>
</template>
