<script setup lang="ts">
import { X } from "@lucide/vue";
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { DIALOG_LAYER_CLASSES } from "../../../../shared/ui/ui-classes";
import SpaceImageComparisonStage from "./SpaceImageComparisonStage.vue";

/**
 * The window a comparison is judged in: opaque and dark in either theme, so neither the canvas
 * behind nor the app's chrome competes with the pictures. The stage mounts with each opening and
 * starts fresh.
 */
const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ before: string | undefined; after: string; error: unknown }>();
const { t } = useI18n();
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogContent
        class="fixed inset-0 bg-neutral-950 text-white outline-none data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
        :class="DIALOG_LAYER_CLASSES.dialog.content"
      >
        <DialogTitle class="sr-only">{{ t("spaces.media.imageComparison.title") }}</DialogTitle>
        <DialogDescription class="sr-only">{{ t("spaces.media.imageComparison.hint") }}</DialogDescription>
        <SpaceImageComparisonStage v-if="props.before" :before="props.before" :after="props.after" />
        <p v-else class="grid size-full place-items-center text-caption text-white/60">
          {{ props.error ? getErrorMessage(props.error) : t("spaces.media.imageComparison.preparing") }}
        </p>
        <DialogClose
          :aria-label="t('common.close')"
          class="absolute top-4 right-4 grid size-9 place-items-center rounded-full bg-neutral-900/85 text-white/80 ring-1 ring-white/10 backdrop-blur-md transition-colors hover:bg-neutral-800 hover:text-white focus-visible:ring-2 focus-visible:ring-white/60 focus-visible:outline-none"
        >
          <X class="size-4.5" :stroke-width="1.8" />
        </DialogClose>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
