<script setup lang="ts">
import { Compass, FileText, Map, Waypoints } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import { useSpaceStore } from "../../session/space-store";

const emit = defineEmits<{ select: [prompt: string] }>();
const { t } = useI18n();
const { activeSpace, nodes } = storeToRefs(useSpaceStore());

const sourceCount = computed(() => nodes.value.filter(isSpaceSemanticNode).length);
const heading = computed(() => {
  const space = activeSpace.value;
  if (!space) return t("spaces.conversation.empty.title");
  if (space.kind === "inbox") return t("spaces.space.inbox");
  return space.title.trim() || t("spaces.conversation.empty.title");
});
const subtitle = computed(() =>
  sourceCount.value > 0
    ? t("spaces.space.nodeCount", { count: sourceCount.value }, sourceCount.value)
    : t("spaces.conversation.empty.emptySpaceSummary"),
);
const prompts = computed(() =>
  sourceCount.value > 0
    ? [
        {
          icon: Map,
          title: t("spaces.conversation.empty.overview"),
          description: t("spaces.conversation.empty.overviewDesc"),
          prompt: t("spaces.conversation.empty.overviewPrompt"),
        },
        {
          icon: FileText,
          title: t("spaces.conversation.empty.brief"),
          description: t("spaces.conversation.empty.briefDesc"),
          prompt: t("spaces.conversation.empty.briefPrompt"),
        },
        {
          icon: Waypoints,
          title: t("spaces.conversation.empty.compare"),
          description: t("spaces.conversation.empty.compareDesc"),
          prompt: t("spaces.conversation.empty.comparePrompt"),
        },
      ]
    : [
        {
          icon: FileText,
          title: t("spaces.conversation.empty.draft"),
          description: t("spaces.conversation.empty.draftDesc"),
          prompt: t("spaces.conversation.empty.draftPrompt"),
        },
        {
          icon: Compass,
          title: t("spaces.conversation.empty.plan"),
          description: t("spaces.conversation.empty.planDesc"),
          prompt: t("spaces.conversation.empty.planPrompt"),
        },
      ],
);
</script>

<template>
  <div class="flex w-full max-w-sm flex-col items-center text-center select-none">
    <UiKnovumAiIcon class="size-5 text-fg-primary/80" />
    <h2 class="mt-2.5 max-w-full truncate text-title font-medium tracking-title text-fg-primary">
      {{ heading }}
    </h2>
    <p class="mt-1 max-w-xs text-caption-sm leading-relaxed text-fg-tertiary tabular-nums">
      {{ subtitle }}
    </p>
    <div class="mt-5 flex flex-wrap items-center justify-center gap-1.5">
      <UiTooltip v-for="item in prompts" :key="item.title" :text="item.description" side="bottom">
        <button
          type="button"
          class="inline-flex h-(--knovum-control-height-sm) items-center gap-1.5 rounded-md border border-stroke-divider bg-surface-raised px-2.5 text-caption text-fg-secondary transition-colors duration-100 hover:border-stroke-default-hover hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
          @click="emit('select', item.prompt)"
        >
          <component
            :is="item.icon"
            class="size-3 shrink-0 text-fg-tertiary"
            :stroke-width="1.65"
            aria-hidden="true"
          />
          <span>{{ item.title }}</span>
        </button>
      </UiTooltip>
    </div>
  </div>
</template>
