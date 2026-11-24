<script setup lang="ts">
import { ArrowUpRight, MessageSquare } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { useSpaceStore } from "../../session/space-store";
import {
  formatRecentConversationTimestamp,
  selectQuickContextItems,
  SPACE_SOURCE_CATEGORY_LABEL_KEY,
  summarizeSpaceSources,
  type SpaceQuickContextItem,
} from "../conversation-empty-guide";
import { buildEmptyConversationStarters, type ConversationStarter } from "../conversation-empty-starters";
import { useConversationSessionStore } from "../conversation-session-store";
import SpaceConversationContextSection from "./SpaceConversationContextSection.vue";

const ROW_CLASS =
  "group -mx-1.5 flex h-interaction w-[calc(100%+0.75rem)] items-center gap-2 rounded-md px-1.5 text-left text-label transition-colors duration-100 hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus";

const emit = defineEmits<{ select: [prompt: string] }>();
const { locale, t } = useI18n();
const { activeSpace, nodes } = storeToRefs(useSpaceStore());
const session = useConversationSessionStore();
const { attachmentCandidates, conversations, showingArchived } = storeToRefs(session);

const inventory = computed(() => summarizeSpaceSources(nodes.value));
const heading = computed(() => {
  const space = activeSpace.value;
  if (!space) return t("spaces.conversation.empty.title");
  if (space.kind === "inbox") return t("spaces.space.inbox");
  return space.title.trim() || t("spaces.conversation.empty.title");
});
const subtitle = computed(() => {
  if (inventory.value.totalCount === 0) return t("spaces.conversation.empty.emptySpaceSummary");
  const total = t(
    "spaces.space.nodeCount",
    { count: inventory.value.totalCount },
    inventory.value.totalCount,
  );
  const breakdown = inventory.value.categories.map(({ category, count }) =>
    t(SPACE_SOURCE_CATEGORY_LABEL_KEY[category], { count }, count),
  );
  return [total, ...breakdown].join(" · ");
});
const recentConversations = computed(() => (showingArchived.value ? [] : conversations.value.slice(0, 3)));
const contextItems = computed(() =>
  selectQuickContextItems(
    attachmentCandidates.value,
    nodes.value,
    recentConversations.value.length > 0 ? 4 : 5,
  ),
);
const selectedTitles = computed(() =>
  attachmentCandidates.value
    .filter(({ selected }) => selected)
    .map(({ title }) => title.trim())
    .filter(Boolean),
);
const starters = computed(() =>
  buildEmptyConversationStarters({
    sourceCount: inventory.value.totalCount,
    selectedTitles: selectedTitles.value,
    maxCount: recentConversations.value.length > 0 ? 4 : 5,
  }),
);

function formatStarterText(key: string, params?: Readonly<Record<string, string | number>>) {
  if (!params) return t(key);
  const count = typeof params.count === "number" ? params.count : undefined;
  return count === undefined ? t(key, params) : t(key, params, count);
}

function chooseStarter(starter: ConversationStarter) {
  if (starter.attachSelection) session.addSelectedAttachments();
  emit("select", formatStarterText(starter.promptKey, starter.params));
}

function toggleContextItem(item: SpaceQuickContextItem) {
  if (item.attached) session.removeAttachment(`node:${item.nodeId}`);
  else session.addNodeAttachments([item.nodeId]);
}

function formatRecentTime(updatedAt: number) {
  return formatRecentConversationTimestamp(
    updatedAt,
    locale.value,
    t("spaces.conversation.conversations.yesterday"),
  );
}
</script>

<template>
  <div class="w-full select-none">
    <header class="pb-3.5">
      <h2 class="truncate text-title-sm font-semibold tracking-title text-fg-primary">
        {{ heading }}
      </h2>
      <p class="mt-0.5 text-caption leading-relaxed text-fg-tertiary tabular-nums">
        {{ subtitle }}
      </p>
    </header>

    <section class="border-t border-stroke-divider/70 py-3">
      <h3 class="mb-1 text-caption font-medium text-fg-tertiary">
        {{ t("spaces.conversation.empty.suggestionsHeading") }}
      </h3>
      <div class="space-y-0.5">
        <UiTooltip
          v-for="item in starters"
          :key="item.id"
          :text="formatStarterText(item.descriptionKey, item.params)"
          side="left"
        >
          <button
            type="button"
            :class="[ROW_CLASS, item.accent ? 'text-fg-primary' : 'text-fg-secondary hover:text-fg-primary']"
            @click="chooseStarter(item)"
          >
            <component
              :is="item.icon"
              class="size-3.5 shrink-0"
              :class="item.accent ? 'text-fg-accent' : 'text-fg-tertiary group-hover:text-fg-primary'"
              :stroke-width="1.6"
              aria-hidden="true"
            />
            <span class="min-w-0 flex-1 truncate">{{ formatStarterText(item.titleKey, item.params) }}</span>
            <ArrowUpRight
              class="size-3.5 shrink-0 text-fg-tertiary opacity-0 transition-opacity duration-100 group-hover:opacity-100 group-focus-visible:opacity-100"
              :stroke-width="1.6"
              aria-hidden="true"
            />
          </button>
        </UiTooltip>
      </div>
    </section>

    <SpaceConversationContextSection
      :items="contextItems"
      :total-count="attachmentCandidates.length"
      @toggle="toggleContextItem"
    />

    <section v-if="recentConversations.length > 0" class="border-t border-stroke-divider/70 py-3">
      <div class="mb-1 flex items-center justify-between gap-2">
        <h3 class="text-caption font-medium text-fg-tertiary">
          {{ t("spaces.conversation.empty.recentHeading") }}
        </h3>
        <span class="text-caption-sm text-fg-tertiary/80 tabular-nums">{{ conversations.length }}</span>
      </div>
      <div class="space-y-0.5">
        <button
          v-for="conversation in recentConversations"
          :key="conversation.id"
          type="button"
          :class="[ROW_CLASS, 'text-fg-secondary hover:text-fg-primary']"
          @click="session.selectConversation(conversation.id)"
        >
          <MessageSquare class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" aria-hidden="true" />
          <span class="min-w-0 flex-1 truncate">{{ conversation.title }}</span>
          <time
            class="shrink-0 text-caption-sm text-fg-tertiary tabular-nums"
            :datetime="new Date(conversation.updatedAt).toISOString()"
          >
            {{ formatRecentTime(conversation.updatedAt) }}
          </time>
        </button>
      </div>
    </section>
  </div>
</template>
