<script setup lang="ts">
import { X } from "@lucide/vue";
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import { computed, type HTMLAttributes } from "vue";
import { useI18n } from "vue-i18n";

import { cn } from "./cn";
import { POPUP_ANIMATE_CLASSES } from "./ui-classes";
import UiIconButton from "./UiIconButton.vue";

const open = defineModel<boolean>("open", { required: true });
const props = withDefaults(
  defineProps<{
    title: string;
    description?: string | undefined;
    size?: "default" | "wide" | undefined;
    closable?: boolean | undefined;
    class?: HTMLAttributes["class"];
    bodyClass?: HTMLAttributes["class"];
    allowedCommandIds?: readonly string[] | undefined;
  }>(),
  {
    description: undefined,
    size: "default",
    closable: true,
    allowedCommandIds: () => [],
  },
);

const { t } = useI18n();
const widthClass = computed(() => (props.size === "wide" ? "max-w-[720px]" : "max-w-md"));
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay
        class="fixed inset-0 z-40 bg-surface-scrim duration-150 ease-standard data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0"
      />
      <DialogContent
        :data-app-command-scope="allowedCommandIds.join(' ')"
        v-bind="description ? {} : { 'aria-describedby': undefined }"
        :class="
          cn(
            POPUP_ANIMATE_CLASSES,
            'fixed top-1/2 left-1/2 z-50 flex max-h-[calc(100vh-5rem)] w-[calc(100vw-2.5rem)] -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-xl border border-stroke-default bg-surface-overlay shadow-dialog duration-150 ease-standard outline-none',
            widthClass,
            props.class,
          )
        "
      >
        <header class="flex min-h-12 shrink-0 items-start gap-3 px-4 pt-4 pb-2">
          <div class="min-w-0 flex-1">
            <DialogTitle class="text-title-sm font-semibold tracking-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="mt-1 text-caption text-fg-secondary">
              {{ description }}
            </DialogDescription>
          </div>
          <DialogClose v-if="closable" as-child>
            <UiIconButton :label="t('common.close')" size="sm"><X class="size-3.5" /></UiIconButton>
          </DialogClose>
        </header>
        <div :class="cn('min-h-0 flex-1 overflow-y-auto px-4 py-3', props.bodyClass)"><slot /></div>
        <footer
          v-if="$slots.footer"
          class="flex shrink-0 justify-end gap-2 border-t border-stroke-default px-4 py-3"
        >
          <slot name="footer" />
        </footer>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
