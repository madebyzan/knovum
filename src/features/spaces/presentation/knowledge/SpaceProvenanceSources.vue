<script setup lang="ts">
import { ExternalLink, Link2, Plus, X } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useContextDock } from "../../canvas/use-context-dock";
import type { SpaceProvenanceTargetKind } from "../../knowledge/model/provenance";
import { knowledgeLocationLabel } from "../../knowledge/presentation/location-label";
import { attachProvenance, detachProvenance } from "../../knowledge/provenance-actions";
import { useSpaceProvenanceQuery } from "../../knowledge/queries/use-knowledge-queries";
import { useSpaceStore } from "../../session/space-store";

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
const attachable = computed(() => {
  const peek = sourcePeek.value;
  return peek && (!props.attachNodeIds || props.attachNodeIds.includes(peek.nodeId)) ? peek : undefined;
});

const nodeTitles = computed(() => new Map(nodes.value.map((node) => [node.id, node.data.title])));

function sourceTitle(nodeId: string) {
  return nodeTitles.value.get(nodeId) ?? nodeId;
}
async function attachCurrent() {
  const peek = attachable.value;
  const spaceId = activeSpaceId.value;
  if (!peek || !spaceId) return;
  await attachProvenance({
    id: crypto.randomUUID(),
    spaceId,
    targetKind: props.targetKind,
    targetId: props.targetId,
    sourceNodeId: peek.nodeId,
    locator: peek.locator,
    sourceRevision: peek.locator.sourceRevision,
    quote: peek.locator.quote?.exact ?? peek.text ?? "",
  });
}
async function remove(id: string) {
  const spaceId = activeSpaceId.value;
  if (!spaceId) return;
  await detachProvenance(id, {
    spaceId,
    targetKind: props.targetKind,
    targetId: props.targetId,
  });
}
function openSource(record: (typeof provenance.value)[number]) {
  dock.openSource(
    {
      nodeId: record.sourceNodeId,
      nodeTitle: sourceTitle(record.sourceNodeId),
      locator: record.locator,
      text: record.quote,
    },
    { reveal: false },
  );
}
</script>

<template>
  <section v-if="provenance.length || attachable" class="rounded-md border border-stroke-divider p-2">
    <div class="flex items-center gap-2">
      <span class="min-w-0 flex-1 text-caption-sm font-medium text-fg-tertiary">{{
        t("spaces.knowledge.provenance")
      }}</span>
      <UiIconButton
        v-if="attachable"
        :label="t('spaces.assistant.evidence.useCurrent')"
        size="sm"
        @click="attachCurrent"
      >
        <Plus class="size-3" />
      </UiIconButton>
    </div>
    <div v-if="provenance.length" class="mt-1.5 space-y-1.5">
      <div v-for="record in provenance" :key="record.id" class="flex items-start gap-1">
        <button
          type="button"
          class="min-w-0 flex-1 rounded-md px-1.5 py-1.5 text-left hover:bg-surface-control-hover"
          @click="openSource(record)"
        >
          <span class="flex items-center gap-1.5 text-caption-md text-fg-primary">
            <Link2 class="size-3 shrink-0 text-fg-tertiary" />
            <span class="truncate">{{ sourceTitle(record.sourceNodeId) }}</span>
            <ExternalLink class="size-3 shrink-0 text-fg-tertiary" />
          </span>
          <span class="mt-0.5 block truncate pl-5 text-caption-sm text-fg-tertiary">
            {{ knowledgeLocationLabel(record.locator) }}
          </span>
          <span v-if="record.quote" class="mt-1 line-clamp-2 block pl-5 text-caption-sm text-fg-secondary">
            “{{ record.quote }}”
          </span>
        </button>
        <UiIconButton
          v-if="removable && record.status === 'active'"
          :label="t('spaces.assistant.evidence.remove')"
          size="sm"
          @click="remove(record.id)"
        >
          <X class="size-3" />
        </UiIconButton>
      </div>
    </div>
  </section>
</template>
