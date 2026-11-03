<script setup lang="ts">
import { ChevronLeft, ChevronRight, Maximize2, ZoomIn, ZoomOut } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { openPdfReader, type PdfReaderSession } from "./pdf-reader-session";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  title: string;
  path: string;
  page?: number | undefined;
  passage?: string | undefined;
}>();
const { t } = useI18n();
const container = useTemplateRef<HTMLDivElement>("container");
const viewer = useTemplateRef<HTMLDivElement>("viewer");
const session = shallowRef<PdfReaderSession>();
const currentPage = shallowRef(1);
const pageCount = shallowRef(0);
const failure = shallowRef<string>();

async function start(containerElement: HTMLDivElement, viewerElement: HTMLDivElement) {
  failure.value = undefined;
  try {
    const opened = await openPdfReader({
      container: containerElement,
      viewer: viewerElement,
      url: convertFileSrc(props.path),
      page: props.page ?? 1,
      passage: props.passage,
      onPageChange(current, total) {
        currentPage.value = current;
        pageCount.value = total;
      },
    });
    // The dialog may have closed while the document loaded; nothing would ever release this one.
    if (open.value) session.value = opened;
    else await opened.close();
  } catch (error) {
    failure.value = getErrorMessage(error);
  }
}

watch([container, viewer], ([containerElement, viewerElement]) => {
  if (containerElement && viewerElement) void start(containerElement, viewerElement);
  else {
    void session.value?.close();
    session.value = undefined;
  }
});
</script>

<template>
  <UiDialog
    v-model:open="open"
    :title="title"
    :description="
      pageCount
        ? t('spaces.inspector.pdfReader.position', { page: currentPage, total: pageCount })
        : undefined
    "
    size="wide"
    class="h-[calc(100vh-3rem)] max-w-6xl"
    body-class="relative min-h-0 flex-1 overflow-hidden p-0"
  >
    <template #actions>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.previousPage')"
        size="sm"
        tooltip
        :disabled="!session || currentPage <= 1"
        @click="session?.goToPage(currentPage - 1)"
      >
        <ChevronLeft class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.nextPage')"
        size="sm"
        tooltip
        :disabled="!session || currentPage >= pageCount"
        @click="session?.goToPage(currentPage + 1)"
      >
        <ChevronRight class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.zoomOut')"
        size="sm"
        tooltip
        :disabled="!session"
        @click="session?.zoom(-1)"
      >
        <ZoomOut class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.zoomIn')"
        size="sm"
        tooltip
        :disabled="!session"
        @click="session?.zoom(1)"
      >
        <ZoomIn class="size-3.5" />
      </UiIconButton>
      <UiIconButton
        :label="t('spaces.inspector.pdfReader.fitWidth')"
        size="sm"
        tooltip
        :disabled="!session"
        @click="session?.fitWidth()"
      >
        <Maximize2 class="size-3.5" />
      </UiIconButton>
    </template>
    <p v-if="failure" class="p-6 text-center text-caption text-fg-danger" role="alert">{{ failure }}</p>
    <!-- pdf.js requires its scroll container to be absolutely positioned. -->
    <div ref="container" class="pdf-reader absolute inset-0 overflow-auto bg-surface-control">
      <div ref="viewer" class="pdfViewer" />
    </div>
  </UiDialog>
</template>

<style scoped>
.pdf-reader :deep(.page) {
  margin: 0 auto 0.75rem;
}
</style>
