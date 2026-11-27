<script setup lang="ts">
import { Minus } from "@lucide/vue";
import type { Editor } from "@tiptap/vue-3";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
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
    <Minus class="size-3.5" :stroke-width="1.6" />
  </UiIconButton>
</template>
