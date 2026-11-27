<script setup lang="ts">
import { Check, Code2, Columns2, Copy, Eye, Maximize2, WrapText } from "@lucide/vue";
import { NodeViewContent, NodeViewWrapper, type NodeViewProps } from "@tiptap/vue-3";
import { computed, onBeforeUnmount, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import SpaceTiptapCodeLanguagePicker from "./SpaceTiptapCodeLanguagePicker.vue";
import SpaceTiptapMermaidPreview from "./SpaceTiptapMermaidPreview.vue";

type MermaidViewMode = "preview" | "split" | "code";

const props = defineProps<NodeViewProps>();
const { t } = useI18n();
const copied = shallowRef(false);
const wrapped = shallowRef(false);
const mermaidMode = shallowRef<MermaidViewMode>(props.node.textContent.trim() ? "preview" : "split");
const mermaidPreviewRef = useTemplateRef<{ openFullscreen: () => void }>("mermaidPreviewRef");
let resetTimer: ReturnType<typeof setTimeout> | undefined;

const currentLanguage = computed(() => {
  const raw = props.node.attrs.language;
  return typeof raw === "string" && raw.trim() ? raw.trim().toLowerCase() : "";
});
const isMermaid = computed(() => currentLanguage.value === "mermaid");

function selectLanguage(value: string) {
  props.updateAttributes({ language: value || null });
  if (value === "mermaid") mermaidMode.value = props.node.textContent.trim() ? "preview" : "split";
}

async function copyCode() {
  try {
    await navigator.clipboard.writeText(props.node.textContent);
    copied.value = true;
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch {
    copied.value = false;
  }
}

onBeforeUnmount(() => clearTimeout(resetTimer));
</script>

<template>
  <NodeViewWrapper
    class="space-code-block group relative my-2 overflow-hidden rounded-panel border border-stroke-default bg-surface-muted transition-colors duration-100 ease-standard hover:border-stroke-strong"
    :data-wrap="wrapped ? 'true' : 'false'"
  >
    <div
      contenteditable="false"
      class="flex items-center justify-between gap-2 px-2.5 select-none"
      :class="isMermaid ? 'h-9 border-b border-stroke-divider' : 'h-8 pt-1'"
    >
      <div class="flex items-center gap-1.5">
        <SpaceTiptapCodeLanguagePicker
          :language="currentLanguage"
          :editable="editor.isEditable"
          @select="selectLanguage"
        />
        <div
          v-if="isMermaid"
          class="inline-flex items-center rounded-md border border-stroke-divider bg-surface-canvas p-0.5"
        >
          <button
            v-for="[mode, icon, labelKey] in [
              ['preview', Eye, 'spaces.richtext.mermaid.preview'],
              ['split', Columns2, 'spaces.richtext.mermaid.split'],
              ['code', Code2, 'spaces.richtext.mermaid.code'],
            ] as const"
            :key="mode"
            type="button"
            :data-testid="`mermaid-mode-${mode}`"
            :aria-pressed="mermaidMode === mode"
            class="inline-flex h-5 items-center gap-1 rounded px-1.5 text-caption-sm transition-colors duration-100 ease-standard"
            :class="
              mermaidMode === mode
                ? 'bg-surface-control font-medium text-fg-primary'
                : 'text-fg-tertiary hover:text-fg-primary'
            "
            @click="mermaidMode = mode"
          >
            <component :is="icon" class="size-3" :stroke-width="1.6" />
            <span>{{ t(labelKey) }}</span>
          </button>
        </div>
      </div>

      <div
        class="flex items-center gap-0.5 opacity-0 transition-opacity duration-100 ease-standard group-focus-within:opacity-100 group-hover:opacity-100"
        :class="(copied || isMermaid) && 'opacity-100'"
      >
        <button
          v-if="isMermaid && mermaidMode !== 'code' && node.textContent.trim()"
          type="button"
          data-testid="mermaid-fullscreen"
          :aria-label="t('spaces.richtext.image.preview')"
          :title="t('spaces.richtext.image.preview')"
          class="grid size-6 place-items-center rounded-md text-fg-tertiary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
          @click="mermaidPreviewRef?.openFullscreen()"
        >
          <Maximize2 class="size-3.5" :stroke-width="1.6" />
        </button>
        <button
          v-if="!isMermaid || mermaidMode !== 'preview'"
          type="button"
          data-testid="code-block-wrap"
          :aria-label="t('spaces.richtext.codeBlock.toggleWrap')"
          :title="t('spaces.richtext.codeBlock.toggleWrap')"
          :aria-pressed="wrapped"
          class="grid size-6 place-items-center rounded-md text-fg-tertiary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus aria-pressed:bg-surface-control aria-pressed:text-fg-primary"
          @click="wrapped = !wrapped"
        >
          <WrapText class="size-3.5" :stroke-width="1.6" />
        </button>
        <button
          type="button"
          data-testid="code-block-copy"
          :aria-label="t('spaces.richtext.codeBlock.copy')"
          :title="copied ? t('spaces.richtext.codeBlock.copied') : t('spaces.richtext.codeBlock.copy')"
          class="inline-flex h-6 items-center gap-1 rounded-md px-1.5 text-caption-sm text-fg-tertiary transition-colors duration-100 ease-standard outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
          @click="copyCode"
        >
          <Check v-if="copied" class="size-3 text-fg-success" :stroke-width="1.8" />
          <Copy v-else class="size-3" :stroke-width="1.6" />
          <span>{{
            copied ? t("spaces.richtext.codeBlock.copied") : t("spaces.richtext.codeBlock.copy")
          }}</span>
        </button>
      </div>
    </div>

    <div
      :class="
        isMermaid &&
        mermaidMode === 'split' &&
        'grid grid-cols-1 divide-y divide-stroke-divider md:grid-cols-2 md:divide-x md:divide-y-0'
      "
    >
      <pre
        class="space-code-pre"
        :class="isMermaid && mermaidMode === 'preview' && 'hidden'"
      ><NodeViewContent as="code" /></pre>
      <SpaceTiptapMermaidPreview
        v-if="isMermaid && mermaidMode !== 'code'"
        ref="mermaidPreviewRef"
        :code="node.textContent"
        @edit="mermaidMode = 'split'"
      />
    </div>
  </NodeViewWrapper>
</template>
