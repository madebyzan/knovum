<script setup lang="ts">
import { Check, ExternalLink, Link2, Trash2 } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Editor } from "@tiptap/vue-3";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import { normalizeUrl } from "../../editor/url-validation";

const LINK_PROTOCOLS = ["http:", "https:", "mailto:"] as const;

const props = defineProps<{ editor: Editor }>();
const { t } = useI18n();
const open = ref(false);
const href = ref("");
const touched = ref(false);
const activeHref = computed(() => String(props.editor.getAttributes("link").href ?? ""));
const normalizedHref = computed(() => normalizeUrl(href.value, LINK_PROTOCOLS));
const invalid = computed(() => touched.value && Boolean(href.value.trim()) && !normalizedHref.value);
const selectedLabel = computed(() => {
  const { from, to } = props.editor.state.selection;
  return from === to ? "" : props.editor.state.doc.textBetween(from, to, " ").trim();
});

watch(open, (value) => {
  if (!value) return;
  href.value = activeHref.value;
  touched.value = false;
});

function apply() {
  touched.value = true;
  if (!normalizedHref.value) return;
  const applied = props.editor
    .chain()
    .focus()
    .extendMarkRange("link")
    .setLink({ href: normalizedHref.value })
    .run();
  if (!applied) {
    toast.error(t("spaces.richtext.link.updateFailed"));
    return;
  }
  open.value = false;
}

function remove() {
  const removed = props.editor.chain().focus().extendMarkRange("link").unsetLink().run();
  if (!removed) {
    toast.error(t("spaces.richtext.link.updateFailed"));
    return;
  }
  open.value = false;
}

async function visit() {
  if (!activeHref.value) return;
  try {
    await openUrl(activeHref.value);
  } catch (error) {
    toast.error(t("spaces.richtext.link.openFailed"), { description: String(error) });
  }
}
</script>

<template>
  <UiPopover
    v-model:open="open"
    align="center"
    :side-offset="7"
    class="w-88 overflow-hidden rounded-dialog p-0"
  >
    <template #trigger>
      <UiIconButton :label="t('spaces.richtext.link.label')" size="sm" :active="editor.isActive('link')">
        <Link2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
      </UiIconButton>
    </template>

    <header class="flex items-start gap-2.5 border-b border-divider px-3 py-2.5">
      <span
        class="mt-0.5 grid size-7 place-items-center rounded-control bg-control text-foreground-secondary"
      >
        <Link2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
      </span>
      <div class="min-w-0 flex-1">
        <div class="text-label font-medium text-foreground">{{ t("spaces.richtext.link.editorTitle") }}</div>
        <p v-if="selectedLabel" class="mt-0.5 truncate text-caption text-foreground-tertiary">
          {{ selectedLabel }}
        </p>
        <p v-else class="mt-0.5 text-caption text-foreground-tertiary">
          {{ t("spaces.richtext.link.editorDescription") }}
        </p>
      </div>
    </header>
    <div class="p-3">
      <label class="grid gap-1.5">
        <span class="text-caption font-medium text-foreground-secondary">{{
          t("spaces.richtext.link.url")
        }}</span>
        <UiInput
          v-model="href"
          autofocus
          type="url"
          :placeholder="t('spaces.richtext.link.placeholder')"
          @input="touched = true"
          @keydown.enter.prevent="apply"
        />
      </label>
      <p v-if="invalid" class="mt-1.5 text-caption text-danger">{{ t("spaces.richtext.link.invalid") }}</p>
    </div>
    <footer class="flex items-center gap-1.5 border-t border-divider px-2.5 py-2">
      <UiButton v-if="activeHref" size="sm" variant="ghost" @click="visit">
        <ExternalLink class="size-(--knovum-icon-size-xs)" />{{ t("spaces.richtext.link.open") }}
      </UiButton>
      <UiButton v-if="activeHref" size="sm" variant="ghost" class="text-danger" @click="remove">
        <Trash2 class="size-(--knovum-icon-size-xs)" />{{ t("spaces.richtext.link.remove") }}
      </UiButton>
      <div class="flex-1" />
      <UiButton size="sm" variant="primary" :disabled="!normalizedHref" @click="apply">
        <Check class="size-(--knovum-icon-size-xs)" />{{ t("common.apply") }}
      </UiButton>
    </footer>
  </UiPopover>
</template>
