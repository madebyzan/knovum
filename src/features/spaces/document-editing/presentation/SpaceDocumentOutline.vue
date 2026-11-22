<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiOutlineRail, { type UiOutlineItem } from "../../../../shared/ui/UiOutlineRail.vue";
import type { SpaceTiptapHeading } from "../tiptap-outline";

const props = defineProps<{ headings: readonly SpaceTiptapHeading[]; activeId?: string | undefined }>();
const emit = defineEmits<{ select: [position: number] }>();
const { t } = useI18n();
const rail = useTemplateRef<InstanceType<typeof UiOutlineRail>>("rail");

const items = computed<readonly UiOutlineItem[]>(() =>
  props.headings.map((heading) => ({ id: heading.id, label: heading.text, level: heading.level })),
);

function selectHeading(id: string) {
  const match = props.headings.find((heading) => heading.id === id);
  if (match) emit("select", match.position);
}

defineExpose({
  focus: () => rail.value?.focus(),
});
</script>

<template>
  <UiOutlineRail
    ref="rail"
    class="top-5 right-4"
    test-id="document-outline"
    :items="items"
    :active-id="activeId"
    :label="t('spaces.sidebar.editorOutline.title')"
    :always-visible="false"
    @select="selectHeading"
  />
</template>
