<script setup lang="ts">
import {
  Boxes,
  Bot,
  ChevronRight,
  CircleHelp,
  FileText,
  FileInput,
  FlaskConical,
  GitBranch,
  Globe2,
  KeyRound,
  Lightbulb,
  LayoutPanelTop,
  LockKeyhole,
  Mic,
  Quote,
  Shapes,
  SlidersHorizontal,
} from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import { SUBMENU_CONTENT_CLASSES } from "../../../shared/ui/ui-classes";
import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../shared/ui/UiSeparator.vue";
import type { SystemKnowledgeObjectKind } from "../knowledge/model/object-types";

const emit = defineEmits<{
  markdown: [];
  knowledge: [kind: SystemKnowledgeObjectKind];
  customKnowledge: [];
  portal: [];
  manageTypes: [];
  website: [];
  importFile: [];
  recordAudio: [];
  credential: [kind: "api-key" | "account-password"];
  agent: [];
  section: [];
}>();
const { t } = useI18n();
const knowledgeOptions = [
  { kind: "concept", icon: Shapes },
  { kind: "question", icon: CircleHelp },
  { kind: "claim", icon: Lightbulb },
  { kind: "evidence", icon: Quote },
  { kind: "experiment", icon: FlaskConical },
  { kind: "decision", icon: GitBranch },
] as const;
</script>

<template>
  <UiDropdownMenuItem @select="emit('markdown')">
    <FileText class="size-(--knovum-icon-size-sm)" />{{ t("spaces.nodeTypes.markdown") }}
  </UiDropdownMenuItem>
  <DropdownMenuSub>
    <DropdownMenuSubTrigger
      class="flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label text-foreground outline-none data-[highlighted]:bg-control-hover"
    >
      <Shapes class="size-(--knovum-icon-size-sm)" />
      <span class="min-w-0 flex-1">{{ t("spaces.knowledge.structuredKnowledge") }}</span>
      <ChevronRight class="size-(--knovum-icon-size-xs) text-foreground-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="SUBMENU_CONTENT_CLASSES">
        <UiDropdownMenuItem
          v-for="option in knowledgeOptions"
          :key="option.kind"
          @select="emit('knowledge', option.kind)"
        >
          <component :is="option.icon" class="size-(--knovum-icon-size-sm)" />{{
            t(`spaces.knowledgeObjectTypes.${option.kind}`)
          }}
        </UiDropdownMenuItem>
        <UiSeparator class="my-1" />
        <UiDropdownMenuItem @select="emit('customKnowledge')">
          <Shapes class="size-(--knovum-icon-size-sm)" />{{ t("spaces.knowledge.objects.custom") }}…
        </UiDropdownMenuItem>
        <UiDropdownMenuItem @select="emit('manageTypes')">
          <SlidersHorizontal class="size-(--knovum-icon-size-sm)" />{{ t("spaces.knowledge.types.manage") }}…
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>

  <UiSeparator class="my-1" />
  <UiDropdownMenuItem @select="emit('section')">
    <LayoutPanelTop class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.sections.createEmpty") }}
  </UiDropdownMenuItem>
  <UiSeparator class="my-1" />
  <UiDropdownMenuItem @select="emit('website')">
    <Globe2 class="size-(--knovum-icon-size-sm)" />{{ t("spaces.importExport.website.add") }}…
  </UiDropdownMenuItem>
  <UiDropdownMenuItem @select="emit('importFile')">
    <FileInput class="size-(--knovum-icon-size-sm)" />{{ t("spaces.canvas.addContent.importFiles") }}
  </UiDropdownMenuItem>
  <UiDropdownMenuItem @select="emit('recordAudio')">
    <Mic class="size-(--knovum-icon-size-sm)" />{{ t("spaces.recording.note") }}…
  </UiDropdownMenuItem>
  <UiSeparator class="my-1" />
  <UiDropdownMenuItem @select="emit('agent')">
    <Bot class="size-(--knovum-icon-size-sm)" />{{ t("spaces.creation.agent.create") }}…
  </UiDropdownMenuItem>
  <UiSeparator class="my-1" />
  <UiDropdownMenuItem @select="emit('portal')">
    <Boxes class="size-(--knovum-icon-size-sm)" />{{ t("spaces.nodeTypes.portal") }}…
  </UiDropdownMenuItem>
  <DropdownMenuSub>
    <DropdownMenuSubTrigger
      class="flex h-(--knovum-compact-item-height) cursor-default items-center gap-2 rounded-control px-2 text-label text-foreground outline-none data-[highlighted]:bg-control-hover"
    >
      <KeyRound class="size-(--knovum-icon-size-sm)" />
      <span class="min-w-0 flex-1">{{ t("spaces.nodeTypes.credential") }}</span>
      <ChevronRight class="size-(--knovum-icon-size-xs) text-foreground-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="SUBMENU_CONTENT_CLASSES">
        <UiDropdownMenuItem @select="emit('credential', 'api-key')">
          <KeyRound class="size-(--knovum-icon-size-sm)" />{{ t("spaces.credentials.apiKey") }}
        </UiDropdownMenuItem>
        <UiDropdownMenuItem @select="emit('credential', 'account-password')">
          <LockKeyhole class="size-(--knovum-icon-size-sm)" />{{ t("spaces.credentials.accountPassword") }}
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
