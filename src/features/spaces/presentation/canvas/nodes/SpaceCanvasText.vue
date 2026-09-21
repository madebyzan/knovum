<script setup lang="ts">
import { EditorContent } from "@tiptap/vue-3";
import { toRef } from "vue";
import { useI18n } from "vue-i18n";

import type { CanvasNodeSizeAuthority } from "../../../canvas/contract/canvas-document";
import type { CanvasContentHeightMeasurement } from "../../../canvas/contract/canvas-editor-port";
import type { SpaceCanvasTextPayload } from "../../../model/nodes/capabilities";
import type { SpaceTextPayload } from "../../../model/nodes/node-payload";
import SpaceCanvasRichText from "./SpaceCanvasRichText.vue";
import SpaceCanvasTextPlane from "./SpaceCanvasTextPlane.vue";
import SpaceCanvasTextSelectionBar from "./SpaceCanvasTextSelectionBar.vue";
import { useCanvasTextEditor } from "./use-canvas-text-editor";

const props = defineProps<{
  id: string;
  data: SpaceTextPayload;
  interactive: boolean;
  verticalClass: string;
  fitContentHeight: (id: string, measurement: CanvasContentHeightMeasurement) => void;
  fontSize: number;
  frameWidth: number;
  textAlign: "left" | "center" | "right";
  persist: (id: string, data: SpaceCanvasTextPayload) => Promise<void>;
  sizeAuthority: CanvasNodeSizeAuthority;
  typographyScale: number;
}>();
const { t } = useI18n();
const placeholder = t("spaces.editor.textPlaceholder");
const { beginEditing, cancelEditing, editing, editor, save } = useCanvasTextEditor({
  id: props.id,
  data: toRef(props, "data"),
  interactive: toRef(props, "interactive"),
  placeholder: "",
  defaultTitle: t("spaces.editor.newText"),
  persist: props.persist,
});

function reportContentHeight(height: number) {
  props.fitContentHeight(props.id, {
    contentRevision: props.data.knowledgeRevision ?? null,
    height,
    sizeAuthority: props.sizeAuthority,
    typographyScale: props.typographyScale,
    width: props.frameWidth,
  });
}
</script>

<template>
  <div
    class="flex size-full min-h-0 flex-col items-center overflow-hidden leading-tight"
    :class="[verticalClass, editing ? 'pointer-events-auto' : 'select-none']"
    :tabindex="interactive ? 0 : undefined"
    @dblclick.stop="beginEditing"
    @keydown.enter="beginEditing"
  >
    <SpaceCanvasTextSelectionBar
      v-if="editor"
      :editor="editor"
      :default-font-size="fontSize"
      :default-alignment="textAlign"
      :typography-scale="typographyScale"
    />
    <SpaceCanvasTextPlane :scale="typographyScale" @content-height="reportContentHeight">
      <EditorContent
        v-if="editing && editor"
        :editor="editor"
        class="w-full overflow-visible"
        @keydown.meta.enter.prevent="save"
        @keydown.ctrl.enter.prevent="save"
        @keydown.escape.prevent="cancelEditing"
      />
      <SpaceCanvasRichText
        v-else
        :content="data.tiptapContent"
        :empty="data.plainText.length === 0"
        kind="text"
        :placeholder="placeholder"
      />
    </SpaceCanvasTextPlane>
  </div>
</template>
