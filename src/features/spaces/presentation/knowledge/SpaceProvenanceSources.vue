<script setup lang="ts">
import { ExternalLink, Link2, Plus, X } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useContextDock } from "../../canvas/use-context-dock";
import { knowledgeLocationLabel } from "../../knowledge/model/location";
import type { SpaceProvenanceTargetKind } from "../../knowledge/model/provenance";
import { attachProvenance, detachProvenance } from "../../knowledge/provenance-actions";
import { useSpaceProvenanceQuery } from "../../knowledge/queries/queries";
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

function sourceTitle(nodeId: string) {
  return nodes.value.find((node) => node.id === nodeId)?.data.title ?? nodeId;
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
function openSource(item: (typeof provenance.value)[number]) {
  dock.openSource(
    {
      nodeId: item.sourceNodeId,
      nodeTitle: sourceTitle(item.sourceNodeId),
      locator: item.locator,
      text: item.quote,
    },
    { reveal: false },
  );
}
</script>

<template>
  <section v-if="provenance.length || attachable" class="rounded-control border border-divider p-2">
    <div class="flex items-center gap-2">
      <span class="min-w-0 flex-1 text-micro font-medium text-foreground-tertiary">{{
        t("spaces.knowledge.provenance")
      }}</span>
      <UiIconButton
        v-if="attachable"
        :label="t('spaces.assistant.useCurrentEvidence')"
        size="sm"
        @click="attachCurrent"
      >
        <Plus class="size-(--knovum-icon-size-xs)" />
      </UiIconButton>
    </div>
    <div v-if="provenance.length" class="mt-1.5 space-y-1.5">
      <div v-for="item in provenance" :key="item.id" class="flex items-start gap-1">
        <button
          type="button"
          class="min-w-0 flex-1 rounded-control px-1.5 py-1.5 text-left hover:bg-control-hover"
          @click="openSource(item)"
        >
          <span class="flex items-center gap-1.5 text-caption text-foreground">
            <Link2 class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
            <span class="truncate">{{ sourceTitle(item.sourceNodeId) }}</span>
            <ExternalLink class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
          </span>
          <span class="mt-0.5 block truncate pl-5 text-micro text-foreground-tertiary">
            {{ knowledgeLocationLabel(item.locator) }}
          </span>
          <span v-if="item.quote" class="mt-1 line-clamp-2 block pl-5 text-micro text-foreground-secondary">
            “{{ item.quote }}”
          </span>
        </button>
        <UiIconButton
          v-if="removable && item.status === 'active'"
          :label="t('spaces.assistant.removeEvidence')"
          size="sm"
          @click="remove(item.id)"
        >
          <X class="size-(--knovum-icon-size-xs)" />
        </UiIconButton>
      </div>
    </div>
  </section>
</template>
