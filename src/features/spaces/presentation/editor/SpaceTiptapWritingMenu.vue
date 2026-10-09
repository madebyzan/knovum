<script setup lang="ts">
import {
  ChevronRight,
  Languages,
  ListCollapse,
  ListPlus,
  MessageSquareMore,
  ScanText,
  Sparkles,
  SpellCheck2,
  WandSparkles,
} from "@lucide/vue";
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRoot,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "reka-ui";
import { useI18n } from "vue-i18n";

import type { SpaceTiptapWritingTools } from "../../editor/use-space-tiptap-writing-tools";

const props = withDefaults(defineProps<{ writing: SpaceTiptapWritingTools; compact?: boolean }>(), {
  compact: false,
});
const { t } = useI18n();
const { runningAction, tones, translations, run, runTranslation } = props.writing;
const itemClass =
  "flex h-9 w-full cursor-default items-center gap-2.5 rounded-md px-2.5 text-left text-label text-fg-primary outline-none transition-colors hover:bg-surface-control-hover data-[highlighted]:bg-surface-control-hover data-[disabled]:pointer-events-none data-[disabled]:opacity-40";
const submenuClass =
  "z-50 w-56 overflow-hidden rounded-xl border border-stroke-default bg-surface-overlay p-1.5 shadow-popover outline-none";
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger as-child>
      <button
        type="button"
        :disabled="Boolean(runningAction)"
        :aria-label="t('spaces.richtext.ai.title')"
        class="group inline-flex h-(--knovum-control-height-sm) items-center gap-1.5 rounded-md px-2 text-label font-medium text-fg-accent transition-colors outline-none hover:bg-surface-selected focus-visible:bg-surface-selected focus-visible:outline-none disabled:opacity-40"
        :class="compact && 'px-2.5'"
      >
        <Sparkles class="size-3.5" :class="runningAction && 'animate-pulse'" :stroke-width="1.7" />
        <span class="whitespace-nowrap">{{
          compact ? t("spaces.richtext.ai.actions.improve") : t("spaces.richtext.ai.title")
        }}</span>
      </button>
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        side="bottom"
        align="start"
        :side-offset="7"
        class="z-50 w-60 rounded-xl border border-stroke-default bg-surface-overlay p-1.5 shadow-popover outline-none"
      >
        <DropdownMenuItem :class="itemClass" @select="run('fix_grammar')">
          <SpellCheck2 class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.fixGrammar") }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('extend')">
          <ListPlus class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.extend") }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('reduce')">
          <ListCollapse class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.reduce") }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('simplify')">
          <ScanText class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.simplify") }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('emojify')">
          <span class="grid size-3.5 place-items-center text-body-lg">🙂</span
          >{{ t("spaces.richtext.ai.actions.emojify") }}
        </DropdownMenuItem>
        <DropdownMenuSeparator class="bg-border-divider my-1 h-px" />
        <DropdownMenuItem :class="itemClass" @select="run('improve')">
          <WandSparkles class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.improve") }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('complete')">
          <Sparkles class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.complete") }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('summarize')">
          <ListCollapse class="size-3.5 text-fg-secondary" />{{ t("spaces.richtext.ai.actions.summarize") }}
        </DropdownMenuItem>
        <DropdownMenuSeparator class="bg-border-divider my-1 h-px" />
        <DropdownMenuSub>
          <DropdownMenuSubTrigger :class="itemClass">
            <MessageSquareMore class="size-3.5 text-fg-secondary" />
            <span class="min-w-0 flex-1">{{ t("spaces.richtext.ai.actions.adjustTone") }}</span>
            <ChevronRight class="size-3 text-fg-tertiary" />
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
              <DropdownMenuItem
                v-for="option in tones"
                :key="option[0]"
                :class="itemClass"
                @select="run(option[0])"
              >
                <span class="size-3.5" />{{ option[1] }}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>
        <DropdownMenuSub>
          <DropdownMenuSubTrigger :class="itemClass">
            <Languages class="size-3.5 text-fg-secondary" />
            <span class="min-w-0 flex-1">{{ t("spaces.richtext.ai.actions.translate") }}</span>
            <ChevronRight class="size-3 text-fg-tertiary" />
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
              <DropdownMenuItem
                v-for="option in translations"
                :key="option[0]"
                :class="itemClass"
                @select="runTranslation(option[0])"
              >
                <span class="size-3.5" />{{ option[1] }}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
