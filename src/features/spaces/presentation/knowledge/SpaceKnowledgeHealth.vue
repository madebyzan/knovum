<script setup lang="ts">
import { AlertTriangle, CheckCircle2, ChevronDown, RefreshCw, ShieldAlert, Wand2 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import { rebuildKnowledgeRetrievalIndex } from "../../knowledge/indexing/rebuild-index";
import type { SpaceKnowledgeHealthIssue } from "../../knowledge/model/health";
import { useSpaceKnowledgeHealthQuery } from "../../knowledge/queries/queries";
import { useSpaceStore } from "../../session/space-store";
import SpaceEnrichmentQueuePanel from "./SpaceEnrichmentQueuePanel.vue";

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

watch(
  () => healthQuery.error.value,
  (error) => {
    if (error) toast.error(String(error));
  },
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
    toast.success(t("spaces.processing.rebuildRetrievalIndexDone", { count: rebuild.indexed }));
    healthQuery.refetch();
  } catch (error) {
    toast.error(String(error));
  } finally {
    rebuilding.value = false;
  }
}
function reveal(issue: SpaceKnowledgeHealthIssue) {
  selectAndRevealSpaceNode(issue.nodeId);
}
</script>

<template>
  <section>
    <div v-if="compact" class="flex items-center gap-0.5">
      <button
        type="button"
        class="flex h-(--knovum-compact-item-height) min-w-0 flex-1 items-center gap-2 rounded-control px-2 text-left text-caption text-foreground-secondary hover:bg-control-hover hover:text-foreground"
        @click="expanded = !expanded"
      >
        <ShieldAlert class="size-(--knovum-icon-size-xs) shrink-0" />
        <span class="min-w-0 flex-1 truncate">{{ t("spaces.knowledge.knowledgeHealth") }}</span>
        <span
          v-if="issues.length"
          class="text-micro tabular-nums"
          :class="criticalCount ? 'text-danger' : 'text-warning'"
        >
          {{ issues.length }}
        </span>
        <CheckCircle2 v-else-if="!loading" class="size-(--knovum-icon-size-xs) text-success" />
        <ChevronDown
          class="size-(--knovum-icon-size-xs) text-foreground-tertiary transition-transform"
          :class="expanded && 'rotate-180'"
        />
      </button>
      <UiIconButton :label="t('common.refresh')" size="sm" :disabled="loading" @click="refresh">
        <RefreshCw class="size-(--knovum-icon-size-xs)" :class="loading && 'animate-spin'" />
      </UiIconButton>
    </div>

    <div
      v-if="!compact || expanded"
      :class="compact && 'mt-1 rounded-panel border border-divider bg-control/15 p-2'"
    >
      <div v-if="!compact" class="flex items-center justify-between gap-2">
        <h3 class="flex items-center gap-1.5 text-caption font-semibold text-foreground-secondary">
          <ShieldAlert class="size-(--knovum-icon-size-xs)" />{{ t("spaces.knowledge.knowledgeHealth") }}
        </h3>
        <UiButton size="sm" variant="ghost" :disabled="loading" @click="refresh">
          <RefreshCw class="size-(--knovum-icon-size-xs)" :class="loading && 'animate-spin'" />{{
            t("common.refresh")
          }}
        </UiButton>
      </div>
      <p v-if="!compact" class="mt-1 text-caption text-foreground-tertiary">
        {{ t("spaces.knowledge.knowledgeHealthDescription") }}
      </p>
      <div
        v-if="!issues.length && !loading"
        class="mt-3 flex items-center gap-2 text-caption text-foreground-secondary"
      >
        <CheckCircle2 class="size-(--knovum-icon-size-sm) text-success" />{{
          t("spaces.knowledge.knowledgeHealthClean")
        }}
      </div>
      <div v-else class="mt-1.5 space-y-1">
        <button
          v-for="issue in issues"
          :key="issue.id"
          type="button"
          class="flex w-full items-start gap-2 rounded-control border border-transparent bg-control/25 p-2 text-left hover:border-divider hover:bg-control-hover"
          @click="reveal(issue)"
        >
          <AlertTriangle
            class="mt-0.5 size-(--knovum-icon-size-xs) shrink-0"
            :class="issue.severity === 'critical' ? 'text-danger' : 'text-warning'"
          />
          <span class="min-w-0">
            <span class="block truncate text-caption font-medium text-foreground">{{ issue.title }}</span>
            <span class="block text-micro text-foreground-tertiary">{{
              t(`spaces.knowledge.knowledgeHealthIssue.${issue.kind}`)
            }}</span>
          </span>
        </button>
      </div>
      <p v-if="criticalCount" class="mt-2 text-micro text-danger">
        {{ t("spaces.knowledge.knowledgeHealthCritical", { count: criticalCount }) }}
      </p>
      <SpaceEnrichmentQueuePanel v-if="activeSpaceId" class="mt-2" :space-id="activeSpaceId" />
      <div v-if="activeSpaceId" class="mt-3 border-t border-divider pt-2">
        <UiButton
          class="w-full justify-start"
          size="sm"
          variant="ghost"
          :disabled="rebuilding"
          @click="rebuildIndex"
        >
          <Wand2 class="size-(--knovum-icon-size-xs)" />{{ t("spaces.processing.rebuildRetrievalIndex") }}
        </UiButton>
      </div>
    </div>
  </section>
</template>
