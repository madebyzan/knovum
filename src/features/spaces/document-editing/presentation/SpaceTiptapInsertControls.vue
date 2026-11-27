<script setup lang="ts">
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { DividerBlockIcon } from "../editor-icons";
import SpaceTiptapImageControl from "./SpaceTiptapImageControl.vue";
import SpaceTiptapTableControl from "./SpaceTiptapTableControl.vue";

defineProps<{ editor: Editor }>();
const { t } = useI18n();
</script>

<template>
  <SpaceTiptapImageControl :editor="editor" />
  <SpaceTiptapTableControl :editor="editor" />
  <UiIconButton
    :label="t('spaces.richtext.horizontalRule')"
    tooltip
    size="sm"
    @click="editor.chain().focus().setHorizontalRule().run()"
  >
    <DividerBlockIcon class="size-3.5" :stroke-width="1.65" />
  </UiIconButton>
</template>
