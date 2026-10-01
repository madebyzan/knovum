<script setup lang="ts">
import { AlertTriangle, CheckCircle2, ChevronDown, RefreshCw, ShieldAlert, Wand2 } from "@lucide/vue";
import { whenever } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import { rebuildKnowledgeRetrievalIndex } from "../../knowledge/indexing/rebuild-index";
import type { SpaceKnowledgeHealthIssue } from "../../knowledge/model/health";
import { useSpaceKnowledgeHealthQuery } from "../../knowledge/queries/use-knowledge-queries";
import { useSpaceStore } from "../../session/space-store";
import SpaceEnrichmentQueuePanel from "../SpaceEnrichmentQueuePanel.vue";

const props = defineProps<{ compact?: boolean }>();
const store = useSpaceStore();
const { activeSpaceId } = storeToRefs(store);
const { t } = useI18n();
const expanded = ref(false);
const rebuilding = ref(false);
const healthQuery = useSpaceKnowledgeHealthQuery(activeSpaceId);
const issues = computed(() => healthQuery.data.value ?? []);
const loading = computed(() => healthQuery.isFetching.value);
const criticalCount = computed(() => issues.value.filter((issue) => issue.severity === "critical").length);

whenever(
  () => healthQuery.error.value,
  (error) => toast.error(errorMessage(error)),
);
function refresh() {
  void healthQuery.refetch();
}
async function rebuildIndex() {
  const spaceId = activeSpaceId.value;
  if (!spaceId || rebuilding.value) return;
  rebuilding.value = true;
  try {
    const rebuild = await rebuildKnowledgeRetrievalIndex(spaceId);
    toast.success(t("spaces.processing.index.rebuildDone", { count: rebuild.indexed }));
    void healthQuery.refetch();
  } catch (error) {
    toast.error(errorMessage(error));
  } finally {
    rebuilding.value = false;
  }
}
function reveal(issue: SpaceKnowledgeHealthIssue) {
  selectAndRevealSpaceNode(issue.nodeId);
}
</script>

<template>
  <CollapsibleRoot as-child :open="!compact || expanded" @update:open="expanded = $event">
    <section>
      <div v-if="compact" class="flex items-center gap-0.5">
        <CollapsibleTrigger
          class="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-2 text-left text-caption text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary"
        >
          <ShieldAlert class="size-3 shrink-0" />
          <span class="min-w-0 flex-1 truncate">{{ t("spaces.knowledge.health.title") }}</span>
          <span
            v-if="issues.length"
            class="text-caption-sm tabular-nums"
            :class="criticalCount ? 'text-fg-danger' : 'text-fg-warning'"
          >
            {{ issues.length }}
          </span>
          <CheckCircle2 v-else-if="!loading" class="size-3 text-fg-success" />
          <ChevronDown
            class="size-3 text-fg-tertiary transition-transform"
            :class="expanded && 'rotate-180'"
          />
        </CollapsibleTrigger>
        <UiIconButton :label="t('common.refresh')" size="sm" :disabled="loading" @click="refresh">
          <RefreshCw class="size-3" :class="loading && 'animate-spin'" />
        </UiIconButton>
      </div>

      <CollapsibleContent
        :class="compact && 'mt-1 rounded-panel border border-stroke-divider bg-surface-control/15 p-2'"
      >
        <div v-if="!compact" class="flex items-center justify-between gap-2">
          <h3 class="flex items-center gap-1.5 text-caption font-semibold text-fg-secondary">
            <ShieldAlert class="size-3" />{{ t("spaces.knowledge.health.title") }}
          </h3>
          <UiButton size="sm" variant="ghost" :disabled="loading" @click="refresh">
            <RefreshCw class="size-3" :class="loading && 'animate-spin'" />{{ t("common.refresh") }}
          </UiButton>
        </div>
        <p v-if="!compact" class="mt-1 text-caption text-fg-tertiary">
          {{ t("spaces.knowledge.health.description") }}
        </p>
        <div
          v-if="!issues.length && !loading"
          class="mt-3 flex items-center gap-2 text-caption text-fg-secondary"
        >
          <CheckCircle2 class="size-3.5 text-fg-success" />{{ t("spaces.knowledge.health.clean") }}
        </div>
        <div v-else class="mt-1.5 space-y-1">
          <button
            v-for="issue in issues"
            :key="issue.id"
            type="button"
            class="flex w-full items-start gap-2 rounded-md border border-transparent bg-surface-control/25 p-2 text-left hover:border-stroke-divider hover:bg-surface-control-hover"
            @click="reveal(issue)"
          >
            <AlertTriangle
              class="mt-0.5 size-3 shrink-0"
              :class="issue.severity === 'critical' ? 'text-fg-danger' : 'text-fg-warning'"
            />
            <span class="min-w-0">
              <span class="block truncate text-caption font-medium text-fg-primary">{{ issue.title }}</span>
              <span class="block text-caption-sm text-fg-tertiary">{{
                t(`spaces.knowledge.health.issues.${issue.kind}`)
              }}</span>
            </span>
          </button>
        </div>
        <p v-if="criticalCount" class="mt-2 text-caption-sm text-fg-danger">
          {{ t("spaces.knowledge.health.critical", { count: criticalCount }) }}
        </p>
        <SpaceEnrichmentQueuePanel v-if="activeSpaceId" class="mt-2" :space-id="activeSpaceId" />
        <div v-if="activeSpaceId" class="mt-3 border-t border-stroke-divider pt-2">
          <UiButton
            class="w-full justify-start"
            size="sm"
            variant="ghost"
            :disabled="rebuilding"
            @click="rebuildIndex"
          >
            <Wand2 class="size-3" />{{ t("spaces.processing.index.rebuild") }}
          </UiButton>
        </div>
      </CollapsibleContent>
    </section>
  </CollapsibleRoot>
</template>
