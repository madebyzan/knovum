<script setup lang="ts">
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { useI18n } from "vue-i18n";

import { useUpdateStore } from "../foundation/updates/update-store";

const updates = useUpdateStore();
const { t } = useI18n();
</script>

<template>
  <DialogRoot :open="updates.blocking">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-50 bg-surface-canvas/80" />
      <DialogContent
        class="fixed inset-0 z-50 flex flex-col items-center justify-center gap-3 p-6"
        @escape-key-down.prevent
        @pointer-down-outside.prevent
        @interact-outside.prevent
        @keydown.stop
      >
        <DialogTitle class="text-body font-semibold">{{
          t("settings.updates.install.installing")
        }}</DialogTitle>
        <DialogDescription class="text-body text-fg-secondary">{{
          t("settings.updates.install.saving")
        }}</DialogDescription>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
