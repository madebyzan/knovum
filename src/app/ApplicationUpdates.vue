<script setup lang="ts">
import { error as logError } from "@tauri-apps/plugin-log";
import { DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from "reka-ui";
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic } from "../foundation/errors/error-format";
import { useTaskStore } from "../foundation/tasks/task-store";
import { updateFailureKeys, updateStatusKeys } from "../foundation/updates/update-presentation";
import { applicationUpdateTaskId as taskId, useUpdateStore } from "../foundation/updates/update-store";
import type { UpdateState } from "../shared/contracts/generated/UpdateState";

const updates = useUpdateStore();
const tasks = useTaskStore();
const router = useRouter();
const { t } = useI18n();
const notifiedVersions = new Set<string>();

onMounted(() => {
  void updates.initialize().catch((error) => {
    void logError(`Updater initialization failed: ${formatErrorDiagnostic(error)}`);
    toast.error(t("settings.updates.action.failed"));
  });
});

function reviewUpdate() {
  void router.push({ path: "/settings", query: { pane: "about" } });
}

function announceUpdate(state: UpdateState) {
  if (state.status !== "available" || notifiedVersions.has(state.release.version)) return;
  notifiedVersions.add(state.release.version);
  toast.info(t("settings.updates.available.title", { version: state.release.version }), {
    action: { label: t("settings.updates.available.review"), onClick: reviewUpdate },
  });
}

function reflectProgress(state: Extract<UpdateState, { status: "downloading" | "verifying" }>) {
  if (!tasks.running.some((task) => task.id === taskId)) {
    tasks.dismiss(taskId);
    tasks.restoreRunning(
      { id: taskId, title: t("settings.updates.title"), startedAt: Date.now() },
      updates.cancelDownload,
    );
    tasks.setOpen(taskId, reviewUpdate);
  }
  const message = t(updateStatusKeys[state.status]);
  if (state.status === "downloading" && state.total) {
    tasks.progress(taskId, (state.downloaded / state.total) * 100, message);
  } else {
    tasks.indeterminate(taskId, message);
  }
  tasks.setCancellation(taskId, state.status === "downloading" ? updates.cancelDownload : undefined);
}

function reflectTask(state: UpdateState) {
  switch (state.status) {
    case "downloading":
    case "verifying":
      reflectProgress(state);
      break;
    case "ready":
      tasks.succeed(taskId);
      break;
    case "failed":
      tasks.fail(taskId, t(updateFailureKeys[state.reason]));
      break;
    case "available":
      tasks.cancelled(taskId);
      break;
  }
}

watch(
  () => updates.state,
  (state) => {
    if (!state) return;
    announceUpdate(state);
    reflectTask(state);
  },
  { immediate: true },
);
</script>

<template>
  <DialogRoot :open="updates.installing" :modal="true">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-50 bg-surface-chrome/90" />
      <DialogContent
        class="fixed inset-0 z-50 flex flex-col items-center justify-center gap-3 text-fg-primary outline-none"
        @escape-key-down.prevent
        @pointer-down-outside.prevent
        @interact-outside.prevent
      >
        <DialogTitle class="font-semibold">{{ t("settings.updates.install.title") }}</DialogTitle>
        <DialogDescription>{{ t("settings.updates.install.description") }}</DialogDescription>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
