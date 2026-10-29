<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import type { KnowledgeChange } from "../../contracts/generated/KnowledgeChange";
import type { KnowledgeProposal } from "../../contracts/generated/KnowledgeProposal";
import { allChanges, toggleChange } from "../proposals/proposal-selection";

/**
 * A proposed Change Set before it reaches the Knowledge graph. Every change can be left out; what
 * depends on it is left out with it, so whatever is accepted stands on its own.
 */
const props = defineProps<{ proposal: KnowledgeProposal; busy: boolean }>();
const emit = defineEmits<{ accept: [changes: number[] | null]; dismiss: [] }>();
const { t } = useI18n();
const selected = shallowRef(allChanges(props.proposal.changes));
watch(
  () => props.proposal.changes,
  (changes) => (selected.value = allChanges(changes)),
);
const titles = computed(() => new Map(props.proposal.nodes.map((node) => [node.id, node.title])));
const createdTitles = computed(() => {
  const created = new Map<string, string>();
  for (const { change } of props.proposal.changes)
    if (change.op === "create-object") created.set(change.tempId, change.title);
  return created;
});
const OUTDATED = {
  "basis-changed": "spaces.knowledge.proposals.freshness.basisChanged",
  "target-gone": "spaces.knowledge.proposals.freshness.targetGone",
} as const;
const current = computed(() => props.proposal.freshness === "current");
const complete = computed(() => selected.value.size === props.proposal.changes.length);

function named(id: string) {
  return createdTitles.value.get(id) ?? titles.value.get(id) ?? id;
}

function describe(change: KnowledgeChange) {
  switch (change.op) {
    case "create-object":
      return change.status === "accepted"
        ? t("spaces.knowledge.proposals.change.claim", { title: change.title })
        : t("spaces.knowledge.proposals.change.draftClaim", { title: change.title });
    case "update-object":
      return t("spaces.knowledge.proposals.change.update", { title: named(change.nodeId) });
    case "create-relation":
      return t("spaces.knowledge.proposals.change.relation", {
        source: named(change.sourceId),
        target: named(change.targetId),
      });
    case "add-provenance":
      return t("spaces.knowledge.proposals.change.evidence", {
        quote: change.quote,
        source: named(change.sourceNodeId),
      });
  }
}

function choose(index: number, chosen: boolean) {
  selected.value = toggleChange(props.proposal.changes, selected.value, index, chosen);
}

function accept() {
  if (!selected.value.size) return toast.error(t("spaces.knowledge.proposals.empty"));
  emit("accept", complete.value ? null : [...selected.value].toSorted((a, b) => a - b));
}
</script>

<template>
  <section class="w-full space-y-2 rounded-panel border border-stroke-divider p-3 text-caption">
    <header class="flex items-baseline justify-between gap-2">
      <h3 class="font-medium text-fg-primary">{{ t("spaces.knowledge.proposals.title") }}</h3>
      <span v-if="proposal.freshness !== 'current'" class="text-fg-danger">
        {{ t(OUTDATED[proposal.freshness]) }}
      </span>
    </header>
    <p v-if="proposal.rationale" class="text-fg-secondary">{{ proposal.rationale }}</p>
    <ul class="divide-y divide-stroke-divider">
      <li v-for="(entry, index) in proposal.changes" :key="index" class="flex gap-2 py-1.5">
        <UiCheckbox
          :model-value="selected.has(index)"
          :label="describe(entry.change)"
          :disabled="busy || !current"
          class="mt-0.5"
          @update:model-value="choose(index, $event)"
        />
        <span class="min-w-0 flex-1 text-fg-primary">{{ describe(entry.change) }}</span>
      </li>
    </ul>
    <footer class="flex justify-end gap-2">
      <UiButton size="sm" variant="ghost" :disabled="busy" @click="emit('dismiss')">
        {{ t("spaces.knowledge.proposals.dismiss") }}
      </UiButton>
      <UiButton size="sm" variant="primary" :disabled="busy || !current" @click="accept">
        {{ t(complete ? "spaces.knowledge.proposals.acceptAll" : "spaces.knowledge.proposals.acceptChosen") }}
      </UiButton>
    </footer>
  </section>
</template>
