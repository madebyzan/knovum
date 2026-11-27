<script setup lang="ts">
import { AlertTriangle, CheckCircle2, ChevronDown, RefreshCw, ShieldAlert } from "@lucide/vue";
import { whenever } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "reka-ui";
import { computed, inject, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { workspaceRightSidebarKey } from "../../../../shared/contracts/workspace-right-sidebar";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import { selectAndRevealSpaceNode } from "../../canvas/selection-actions";
import { useSpaceStore } from "../../session/space-store";
import type { SpaceKnowledgeHealthIssue } from "../model/health";
import { useSpaceKnowledgeHealthQuery } from "../queries/use-space-knowledge-health-query";

const props = defineProps<{ compact?: boolean; inDialog?: boolean }>();
const store = useSpaceStore();
const { activeSpaceId } = storeToRefs(store);
const { t } = useI18n();
const rightSidebar = inject(workspaceRightSidebarKey, undefined);
const expanded = ref(false);
const { data, isFetching: loading, error: queryError, refetch } = useSpaceKnowledgeHealthQuery(activeSpaceId);
const issues = computed(() => data.value ?? []);
const criticalCount = computed(() => issues.value.filter((issue) => issue.severity === "critical").length);

whenever(queryError, (cause) => toast.error(getErrorMessage(cause)));
function reveal(issue: SpaceKnowledgeHealthIssue) {
  selectAndRevealSpaceNode(issue.nodeId);
  store.knowledgeHealthOpen = false;
}
function askAboutIssue(issue: SpaceKnowledgeHealthIssue) {
  selectAndRevealSpaceNode(issue.nodeId);
  store.knowledgeHealthOpen = false;
  if (!store.workbench.rightSidebar) return;
  rightSidebar?.open();
  store.requestAsk("selection", { nodeIds: [issue.nodeId] });
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
        <UiIconButton :label="t('common.refresh')" size="sm" :disabled="loading" @click="refetch()">
          <RefreshCw class="size-3" :class="loading && 'animate-spin'" />
        </UiIconButton>
      </div>

      <CollapsibleContent
        :class="compact && 'mt-1 rounded-panel border border-stroke-divider bg-surface-control/15 p-2'"
      >
        <div v-if="!compact && !inDialog" class="flex items-center justify-between gap-2">
          <h3 class="flex items-center gap-1.5 text-caption font-semibold text-fg-secondary">
            <ShieldAlert class="size-3" />{{ t("spaces.knowledge.health.title") }}
          </h3>
          <UiIconButton :label="t('common.refresh')" size="sm" :disabled="loading" @click="refetch()">
            <RefreshCw class="size-3" :class="loading && 'animate-spin'" />
          </UiIconButton>
        </div>
        <p v-if="!compact && !inDialog" class="mt-1 text-caption text-fg-tertiary">
          {{ t("spaces.knowledge.health.description") }}
        </p>
        <div
          v-if="!issues.length && !loading"
          class="flex items-center justify-between gap-2 text-caption text-fg-secondary"
          :class="!inDialog && 'mt-3'"
        >
          <span class="flex items-center gap-2">
            <CheckCircle2 class="size-3.5 shrink-0 text-fg-success" />
            <span>{{ t("spaces.knowledge.health.clean") }}</span>
          </span>
          <UiIconButton
            v-if="inDialog"
            :label="t('common.refresh')"
            size="sm"
            :disabled="loading"
            @click="refetch()"
          >
            <RefreshCw class="size-3" :class="loading && 'animate-spin'" />
          </UiIconButton>
        </div>
        <template v-else>
          <div v-if="inDialog" class="mb-2 flex items-center justify-between gap-2">
            <h3 class="flex items-center gap-1.5 text-caption font-semibold text-fg-secondary">
              <ShieldAlert class="size-3" />{{ t("spaces.knowledge.health.title") }}
            </h3>
            <UiIconButton :label="t('common.refresh')" size="sm" :disabled="loading" @click="refetch()">
              <RefreshCw class="size-3" :class="loading && 'animate-spin'" />
            </UiIconButton>
          </div>
          <div class="space-y-1" :class="!inDialog && 'mt-1.5'">
            <div
              v-for="issue in issues"
              :key="issue.id"
              class="group flex items-center gap-1 rounded-md border border-transparent bg-surface-control/25 pr-1 hover:border-stroke-divider hover:bg-surface-control-hover"
            >
              <button
                type="button"
                class="flex min-w-0 flex-1 items-start gap-2 rounded-md p-2 text-left"
                @click="reveal(issue)"
              >
                <AlertTriangle
                  class="mt-0.5 size-3 shrink-0"
                  :class="issue.severity === 'critical' ? 'text-fg-danger' : 'text-fg-warning'"
                />
                <span class="min-w-0 flex-1">
                  <span class="block truncate text-caption font-medium text-fg-primary">{{
                    issue.title
                  }}</span>
                  <span class="block text-caption text-fg-tertiary">{{
                    t(`spaces.knowledge.health.issues.${issue.kind}`)
                  }}</span>
                </span>
              </button>
              <UiIconButton
                v-if="store.workbench.rightSidebar"
                :label="t('spaces.conversation.askSelection')"
                tooltip
                size="sm"
                class="shrink-0 text-fg-secondary opacity-0 transition-opacity group-hover:opacity-100 hover:text-fg-primary focus-visible:opacity-100"
                @click.stop="askAboutIssue(issue)"
              >
                <UiKnovumAiIcon class="size-3.5" />
              </UiIconButton>
            </div>
          </div>
        </template>
        <p v-if="criticalCount" class="mt-2 text-caption-sm text-fg-danger">
          {{ t("spaces.knowledge.health.critical", { count: criticalCount }) }}
        </p>
      </CollapsibleContent>
    </section>
  </CollapsibleRoot>
</template>
