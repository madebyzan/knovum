<script setup lang="ts">
import { Search } from "@lucide/vue";
import { whenever } from "@vueuse/core";
import { search } from "fast-fuzzy";
import {
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  ListboxContent,
  ListboxFilter,
  ListboxItem,
  ListboxRoot,
} from "reka-ui";
import { computed, nextTick, ref } from "vue";
import { useI18n } from "vue-i18n";

import { runAppCommand } from "../../foundation/commands/run-app-command";
import { canRunAction } from "../../shared/contracts/action-availability";
import type { AppCommand } from "../../shared/contracts/app-command";
import UiKbd from "../../shared/ui/UiKbd.vue";
import UiTooltip from "../../shared/ui/UiTooltip.vue";
import { commandUnavailableLabelKeys } from "../command-unavailable-labels";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ commands: readonly AppCommand[] }>();
const { t } = useI18n();

const query = ref("");
whenever(
  () => !open.value,
  () => {
    query.value = "";
  },
);

const filteredCommands = computed(() => {
  const visible = props.commands.filter(
    (command) => command.palette !== false && command.availability?.().state !== "hidden",
  );
  const needle = query.value.trim();
  if (!needle) return visible;
  return search(needle, visible, {
    keySelector: (cmd) => `${cmd.title} ${cmd.description ?? ""}`,
    threshold: 0.6,
  });
});

async function handleSelect(value: unknown) {
  const command = value as AppCommand | undefined;
  if (!command) return;
  if (!commandReady(command)) return;
  open.value = false;
  await nextTick();
  runAppCommand(command);
}

function commandReady(command: AppCommand) {
  return !command.availability || canRunAction(command.availability());
}

function commandReason(command: AppCommand) {
  const availability = command.availability?.();
  if (availability?.state === "blocked")
    return t(`shell.commandUnavailable.${commandUnavailableLabelKeys[availability.reason]}`);
  if (availability?.state === "checking") return t("shell.commandUnavailable.checking");
  if (availability?.state === "busy") return t("shell.commandUnavailable.busy");
  return "";
}
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-surface-scrim" />
      <DialogContent
        data-app-command-scope=""
        class="fixed top-[16%] left-1/2 z-50 w-[calc(100vw-3rem)] max-w-156 -translate-x-1/2 overflow-hidden rounded-xl border border-stroke-default material-overlay shadow-dialog outline-none"
      >
        <DialogTitle class="sr-only">{{ t("shell.commands.title") }}</DialogTitle>
        <ListboxRoot @update:model-value="handleSelect">
          <div class="flex h-12 items-center gap-2.5 border-b border-stroke-divider px-3.5">
            <Search class="size-3.5 text-fg-tertiary" />
            <ListboxFilter
              v-model="query"
              data-testid="command-palette-input"
              autofocus
              :placeholder="t('shell.commands.searchPlaceholder')"
              class="h-full min-w-0 flex-1 border-0 bg-transparent text-body-lg outline-none placeholder:text-fg-tertiary"
            />
            <UiKbd shortcut="esc" />
          </div>
          <ListboxContent class="max-h-90 space-y-0.5 overflow-y-auto p-1.5">
            <UiTooltip
              v-for="command in filteredCommands"
              :key="command.id"
              :text="commandReason(command)"
              :disabled="commandReady(command)"
              side="right"
            >
              <ListboxItem
                :value="command"
                :aria-disabled="!commandReady(command)"
                class="flex h-(--knovum-command-palette-item-height) w-full cursor-default items-center gap-2.5 rounded-md px-2.5 text-left outline-none aria-disabled:opacity-40 data-highlighted:bg-(--knovum-sidebar-item-bg-selected)"
              >
                <span class="grid size-6 place-items-center text-fg-secondary">
                  <component :is="command.icon" v-if="command.icon" class="size-3.5" />
                </span>
                <span class="min-w-0 flex-1">
                  <strong class="block truncate text-label font-medium">{{ command.title }}</strong>
                  <span v-if="command.description" class="block truncate text-caption text-fg-tertiary">{{
                    command.description
                  }}</span>
                </span>
                <UiKbd v-if="command.shortcut" :shortcut="command.shortcut" />
              </ListboxItem>
            </UiTooltip>
            <div v-if="!filteredCommands.length" class="py-8 text-center text-label text-fg-tertiary">
              {{ t("shell.commands.noMatches") }}
            </div>
          </ListboxContent>
        </ListboxRoot>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
