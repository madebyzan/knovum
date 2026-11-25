<script setup lang="ts">
import { ArrowUpCircle, RotateCw } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic } from "../../foundation/errors/error-format";
import {
  getUpdateRelease,
  isUpdateToolbarVisible,
  updateTransfer,
} from "../../foundation/updates/update-presentation";
import { useUpdateStore } from "../../foundation/updates/update-store";
import { isApplicationLeaveCancelled } from "../../shared/contracts/application-leave";
import UiPopover from "../../shared/ui/UiPopover.vue";
import TaskActivityIndicator from "../tasks/TaskActivityIndicator.vue";
import WorkspaceUpdatePopover from "./WorkspaceUpdatePopover.vue";

const emit = defineEmits<{ openSettings: [] }>();
const updates = useUpdateStore();
const { t, locale } = useI18n();
const open = shallowRef(false);

const state = computed(() => updates.state);
const visible = computed(() => isUpdateToolbarVisible(state.value));
const release = computed(() => getUpdateRelease(state.value));
const transfer = computed(() => (state.value ? updateTransfer(state.value, locale.value) : undefined));
const progressPercent = computed(() => {
  const current = transfer.value;
  if (!current || current.value === null) return undefined;
  return (current.value / current.max) * 100;
});

const triggerLabel = computed(() => {
  const current = state.value;
  if (!current) return "";
  if (current.status === "ready") return t("updates.indicator.ready");
  if (current.status === "downloading") return transfer.value?.percent ?? t("updates.indicator.downloading");
  if (current.status === "verifying") return t("updates.indicator.verifying");
  return t("updates.indicator.available");
});

const triggerTitle = computed(() => {
  const version = release.value?.version;
  if (!version) return t("updates.title");
  return state.value?.status === "ready"
    ? t("updates.ready.title", { version })
    : t("updates.available.title", { version });
});

const triggerClasses = computed(() => {
  if (state.value?.status === "ready") {
    return "h-(--knovum-control-height-sm) bg-surface-inverse px-2.5 text-fg-inverse hover:bg-surface-inverse-hover";
  }
  return "h-(--knovum-control-height-md) px-2 text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary data-[state=open]:bg-surface-control-hover data-[state=open]:text-fg-primary";
});

watch(visible, (shown) => {
  if (!shown) open.value = false;
});
watch(
  () => updates.installing,
  (installing) => {
    if (installing) open.value = false;
  },
);

function openUpdateSettings() {
  open.value = false;
  emit("openSettings");
}

async function perform(action: () => Promise<unknown>) {
  try {
    await action();
  } catch (error) {
    if (isApplicationLeaveCancelled(error)) return;
    toast.error(t("updates.action.failed"));
    void logError(`Application update action failed: ${formatErrorDiagnostic(error)}`);
  }
}
</script>

<template>
  <UiPopover
    v-if="visible && state && release"
    v-model:open="open"
    align="end"
    side="bottom"
    :side-offset="7"
    class="w-86 overflow-hidden rounded-xl p-0"
  >
    <template #trigger>
      <button
        type="button"
        data-testid="workspace-update-indicator"
        :aria-label="triggerTitle"
        :title="triggerTitle"
        class="inline-flex items-center justify-center gap-1.5 rounded-md text-caption font-medium transition-colors duration-100 ease-standard outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus motion-reduce:transition-none"
        :class="triggerClasses"
      >
        <RotateCw v-if="state.status === 'ready'" class="size-3 shrink-0" :stroke-width="1.9" />
        <TaskActivityIndicator
          v-else-if="state.status === 'downloading' || state.status === 'verifying'"
          variant="ring"
          :progress="progressPercent"
        />
        <ArrowUpCircle v-else class="size-3.5 shrink-0 text-fg-accent" :stroke-width="1.75" />
        <span :class="state.status === 'available' ? 'text-fg-primary tabular-nums' : 'tabular-nums'">
          {{ triggerLabel }}
        </span>
      </button>
    </template>

    <WorkspaceUpdatePopover
      :state="state"
      :release="release"
      :request-pending="updates.requestPending"
      :blocked-by-tasks="updates.blockedByTasks"
      :can-install="updates.canInstall"
      @open-settings="openUpdateSettings"
      @download="perform(updates.download)"
      @cancel="perform(updates.cancelDownload)"
      @install="perform(updates.install)"
    />
  </UiPopover>
</template>
