import { storeToRefs } from "pinia";
import { computed, onMounted, reactive, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../session/space-store";
import { listWorkflowVersions, type WorkflowVersion } from "../workflows/workflow-client";
import {
  deleteAutomationTrigger,
  listAutomationOccurrences,
  listAutomationTriggers,
  previewAutomationSchedule,
  renewAutomationTrigger,
  saveAutomationTrigger,
  setAutomationTriggerEnabled,
  testAutomationTrigger,
  type AutomationOccurrence,
  type AutomationTrigger,
} from "./automation-client";

export function useAutomationCenter() {
  const { t } = useI18n();
  const { activeSpaceId } = storeToRefs(useSpaceStore());
  const triggers = shallowRef<AutomationTrigger[]>([]);
  const versions = shallowRef<WorkflowVersion[]>([]);
  const occurrences = shallowRef<AutomationOccurrence[]>([]);
  const error = shallowRef<string>();
  const saving = shallowRef(false);
  const schedulePreview = shallowRef<number[]>([]);
  const form = reactive({
    name: "",
    workflow: "",
    sourceKind: "cron" as "cron" | "event",
    schedule: "0 0 9 * * *",
    eventKind: "dev.knovum.space.action.create-note.committed.v1",
    filterExpression: "",
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    overlapPolicy: "skip" as "skip" | "queue" | "parallel" | "cancel_previous",
    misfirePolicy: "latest" as "skip" | "latest" | "catch_up",
  });
  const workflowOptions = computed(() =>
    versions.value.map((version) => ({
      value: `${version.workflowId}:${version.version}`,
      label: `${version.definition.title} · v${version.version}`,
    })),
  );
  const overlapMessageKeys = {
    skip: "spaces.assistant.overlap.skip",
    queue: "spaces.assistant.overlap.queue",
    parallel: "spaces.assistant.overlap.parallel",
    cancel_previous: "spaces.assistant.overlap.cancelPrevious",
  } as const;
  const misfireMessageKeys = {
    skip: "spaces.assistant.misfire.skip",
    latest: "spaces.assistant.misfire.latest",
    catch_up: "spaces.assistant.misfire.catchUp",
  } as const;
  const sourceMessageKeys = {
    cron: "spaces.assistant.triggerSource.cron",
    event: "spaces.assistant.triggerSource.event",
  } as const;
  const options = <Value extends string>(messageKeys: Record<Value, string>) =>
    Object.entries(messageKeys).map(([value, key]) => ({ value: value as Value, label: t(key as string) }));
  const overlapOptions = computed(() => options(overlapMessageKeys));
  const misfireOptions = computed(() => options(misfireMessageKeys));
  const sourceOptions = computed(() => options(sourceMessageKeys));

  async function refresh() {
    if (!activeSpaceId.value) return;
    [triggers.value, versions.value, occurrences.value] = await Promise.all([
      listAutomationTriggers(activeSpaceId.value),
      listWorkflowVersions(activeSpaceId.value),
      listAutomationOccurrences(activeSpaceId.value),
    ]);
    if (!form.workflow && workflowOptions.value[0]) form.workflow = workflowOptions.value[0].value;
  }
  async function save() {
    if (!activeSpaceId.value || !form.name.trim() || !form.workflow) return;
    const [workflowId, version] = form.workflow.split(":");
    if (!workflowId || !version) return;
    saving.value = true;
    error.value = undefined;
    try {
      await saveAutomationTrigger({
        id: crypto.randomUUID(),
        spaceId: activeSpaceId.value,
        name: form.name.trim(),
        workflowId,
        workflowVersion: Number(version),
        input: {},
        overlapPolicy: form.overlapPolicy,
        source: source(),
        enabled: true,
        validUntil: renewalTime(),
      });
      form.name = "";
      await refresh();
    } catch (reason) {
      error.value = String(reason);
    } finally {
      saving.value = false;
    }
  }
  function source() {
    return form.sourceKind === "cron"
      ? {
          kind: "cron" as const,
          schedule: form.schedule,
          timezone: form.timezone,
          misfirePolicy: form.misfirePolicy,
          catchUpLimit: 10,
          misfireWindowSeconds: 86_400,
        }
      : {
          kind: "event" as const,
          eventKind: form.eventKind,
          filterExpression: form.filterExpression || null,
          causalDepthLimit: 8,
          rateLimitCount: 20,
          rateLimitWindowSeconds: 60,
        };
  }
  async function previewSchedule() {
    error.value = undefined;
    try {
      schedulePreview.value = await previewAutomationSchedule(form.schedule, form.timezone);
    } catch (reason) {
      error.value = String(reason);
    }
  }
  const renew = async (trigger: AutomationTrigger) => {
    await renewAutomationTrigger(trigger.id, renewalTime());
    await refresh();
  };
  const toggle = async (trigger: AutomationTrigger) => {
    await setAutomationTriggerEnabled(trigger.id, !trigger.enabled);
    await refresh();
  };
  const test = async (trigger: AutomationTrigger) => {
    await testAutomationTrigger(trigger.id);
    await refresh();
  };
  const remove = async (trigger: AutomationTrigger) => {
    await deleteAutomationTrigger(trigger.id);
    await refresh();
  };
  watch(activeSpaceId, refresh);
  onMounted(refresh);
  return {
    error,
    form,
    misfireOptions,
    occurrences,
    overlapOptions,
    previewSchedule,
    remove,
    renew,
    save,
    saving,
    schedulePreview,
    sourceLabel,
    sourceOptions,
    test,
    toggle,
    triggers,
    workflowOptions,
  };
}

const renewalTime = () => Date.now() + 365 * 24 * 60 * 60 * 1000;
const sourceLabel = (trigger: AutomationTrigger) =>
  trigger.source.kind === "cron"
    ? `${trigger.source.schedule} · ${trigger.source.timezone}`
    : trigger.source.eventKind;
