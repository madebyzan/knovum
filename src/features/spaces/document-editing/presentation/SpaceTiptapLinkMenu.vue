<script setup lang="ts">
import { Check, Copy, ExternalLink, Link2, Pencil, Unlink2 } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Editor as CoreEditor } from "@tiptap/core";
import type { Editor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";
import { computed, onBeforeUnmount, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { POPOVER_SURFACE_CLASSES } from "../../../../shared/ui/ui-classes";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import { normalizeUrl } from "../url-validation";
import type { SpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";

const LINK_PROTOCOLS = ["http:", "https:", "mailto:"] as const;

const props = defineProps<{ editor: Editor; writing: SpaceTiptapWritingTools }>();
const { t } = useI18n();
const editing = shallowRef(false);
const draftHref = shallowRef("");
const copied = shallowRef(false);
let resetTimer: ReturnType<typeof setTimeout> | undefined;

const activeHref = computed(() => String(props.editor.getAttributes("link").href ?? ""));
const normalizedHref = computed(() => normalizeUrl(draftHref.value, LINK_PROTOCOLS));

function shouldShowLinkMenu({ editor }: { editor: CoreEditor }) {
  const show =
    !editor.isDestroyed &&
    editor.isEditable &&
    props.writing.phase.value === "closed" &&
    editor.isActive("link") &&
    editor.state.selection.empty;
  if (!show) editing.value = false;
  return show;
}

function beginEdit() {
  draftHref.value = activeHref.value;
  editing.value = true;
}

function applyEdit() {
  if (!normalizedHref.value) return;
  props.editor.chain().focus().extendMarkRange("link").setLink({ href: normalizedHref.value }).run();
  editing.value = false;
}

async function copyHref() {
  if (!activeHref.value) return;
  try {
    await navigator.clipboard.writeText(activeHref.value);
    copied.value = true;
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => {
      copied.value = false;
    }, 1400);
  } catch {
    copied.value = false;
  }
}

async function visitHref() {
  const safe = normalizeUrl(activeHref.value, LINK_PROTOCOLS);
  if (!safe) return;
  try {
    await openUrl(safe);
  } catch (error) {
    toast.error(t("spaces.richtext.link.openFailed"), { description: getErrorMessage(error) });
  }
}

onBeforeUnmount(() => clearTimeout(resetTimer));
</script>

<template>
  <BubbleMenu :editor="editor" plugin-key="linkBubbleMenu" :should-show="shouldShowLinkMenu">
    <div
      data-testid="tiptap-link-menu"
      :class="[POPOVER_SURFACE_CLASSES, 'z-40 flex items-center gap-1 px-2 py-1']"
    >
      <template v-if="editing">
        <UiInput
          v-model="draftHref"
          autofocus
          type="url"
          class="h-7 w-56 text-caption"
          :placeholder="t('spaces.richtext.link.placeholder')"
          @commit.prevent="applyEdit"
          @keydown.escape.prevent="editing = false"
        />
        <UiButton size="sm" variant="primary" :disabled="!normalizedHref" @click="applyEdit">
          {{ t("common.apply") }}
        </UiButton>
      </template>
      <template v-else>
        <Link2 class="ml-0.5 size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.65" />
        <button
          type="button"
          class="max-w-52 truncate rounded-sm px-1 py-0.5 text-left text-caption text-fg-accent hover:underline"
          :title="activeHref"
          @click="visitHref"
        >
          {{ activeHref }}
        </button>
        <UiIconButton :label="t('spaces.richtext.link.open')" size="sm" @click="visitHref">
          <ExternalLink class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiIconButton :label="t('spaces.richtext.link.copy')" size="sm" @click="copyHref">
          <Check v-if="copied" class="size-3.5 text-fg-success" :stroke-width="1.9" />
          <Copy v-else class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiIconButton :label="t('spaces.richtext.link.edit')" size="sm" @click="beginEdit">
          <Pencil class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
        <UiIconButton
          :label="t('spaces.richtext.link.remove')"
          size="sm"
          class="text-fg-danger"
          @click="editor.chain().focus().extendMarkRange('link').unsetLink().run()"
        >
          <Unlink2 class="size-3.5" :stroke-width="1.6" />
        </UiIconButton>
      </template>
    </div>
  </BubbleMenu>
</template>
