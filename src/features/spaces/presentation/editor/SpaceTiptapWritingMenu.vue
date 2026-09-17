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
import type { Editor } from "@tiptap/vue-3";
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

import { useSpaceTiptapWritingTools } from "../../editor/tiptap-writing-tools";

const props = withDefaults(defineProps<{ editor: Editor; compact?: boolean }>(), { compact: false });
const { t } = useI18n();
const { runningAction, itemClass, submenuClass, tones, translations, run, runTranslation } =
  useSpaceTiptapWritingTools(props.editor);
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger as-child>
      <button
        type="button"
        :disabled="Boolean(runningAction)"
        :aria-label="t('editor.ai')"
        class="group inline-flex h-(--knovum-control-height-sm) items-center gap-1.5 rounded-control px-2 text-label font-medium text-accent transition-colors outline-none hover:bg-selected focus-visible:bg-selected focus-visible:outline-none disabled:opacity-45"
        :class="compact && 'px-2.5'"
      >
        <Sparkles
          class="size-(--knovum-icon-size-sm)"
          :class="runningAction && 'animate-pulse'"
          :stroke-width="1.7"
        />
        <span class="whitespace-nowrap">{{ compact ? t("editor.aiAction.improve") : t("editor.ai") }}</span>
      </button>
    </DropdownMenuTrigger>
    <DropdownMenuPortal>
      <DropdownMenuContent
        side="bottom"
        align="start"
        :side-offset="7"
        class="z-50 w-60 rounded-dialog border border-border bg-overlay p-1.5 shadow-popover outline-none"
      >
        <DropdownMenuItem :class="itemClass" @select="run('fix_grammar')">
          <SpellCheck2 class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.fixGrammar")
          }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('extend')">
          <ListPlus class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.extend")
          }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('reduce')">
          <ListCollapse class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.reduce")
          }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('simplify')">
          <ScanText class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.simplify")
          }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('emojify')">
          <span class="grid size-(--knovum-icon-size-sm) place-items-center text-body-lg">🙂</span
          >{{ t("editor.aiAction.emojify") }}
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1 h-px bg-divider" />
        <DropdownMenuItem :class="itemClass" @select="run('improve')">
          <WandSparkles class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.improve")
          }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('complete')">
          <Sparkles class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.complete")
          }}
        </DropdownMenuItem>
        <DropdownMenuItem :class="itemClass" @select="run('summarize')">
          <ListCollapse class="size-(--knovum-icon-size-sm) text-foreground-secondary" />{{
            t("editor.aiAction.summarize")
          }}
        </DropdownMenuItem>
        <DropdownMenuSeparator class="my-1 h-px bg-divider" />
        <DropdownMenuSub>
          <DropdownMenuSubTrigger :class="itemClass">
            <MessageSquareMore class="size-(--knovum-icon-size-sm) text-foreground-secondary" />
            <span class="min-w-0 flex-1">{{ t("editor.aiAction.adjustTone") }}</span>
            <ChevronRight class="size-(--knovum-icon-size-xs) text-foreground-tertiary" />
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
              <DropdownMenuItem
                v-for="option in tones"
                :key="option[0]"
                :class="itemClass"
                @select="run(option[0])"
              >
                <span class="size-(--knovum-icon-size-sm)" />{{ option[1] }}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>
        <DropdownMenuSub>
          <DropdownMenuSubTrigger :class="itemClass">
            <Languages class="size-(--knovum-icon-size-sm) text-foreground-secondary" />
            <span class="min-w-0 flex-1">{{ t("editor.aiAction.translate") }}</span>
            <ChevronRight class="size-(--knovum-icon-size-xs) text-foreground-tertiary" />
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
              <DropdownMenuItem
                v-for="option in translations"
                :key="option[0]"
                :class="itemClass"
                @select="runTranslation(option[0])"
              >
                <span class="size-(--knovum-icon-size-sm)" />{{ option[1] }}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
