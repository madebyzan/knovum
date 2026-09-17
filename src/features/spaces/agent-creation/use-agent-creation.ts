import { storeToRefs } from "pinia";
import { computed, onBeforeUnmount, ref, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import type { AgentKind } from "../../../foundation/ai/agent-models";
import { listRecoverableAgentCreations, runAgentCreation } from "../../../foundation/ai/agents";
import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../foundation/ai/execution-catalog-query";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useSpaceStore } from "../session/space-store";
import { useAgentArtifacts } from "./use-agent-artifacts";

export function useAgentCreation(open: Ref<boolean>) {
  const { t } = useI18n();
  const router = useRouter();
  const store = useSpaceStore();
  const tasks = useTaskStore();
  const aiStore = useAiSettingsStore();
  const { settings } = storeToRefs(aiStore);
  const catalog = useExecutionCatalogQuery();
  const artifactState = useAgentArtifacts(open);
  const selectedAgent = ref("");
  const prompt = ref("");
  const running = ref(false);
  const status = ref("");
  let abandonRequested = false;

  const agents = computed(() => {
    const enabled = new Set(settings.value.desktopAgents.map(({ kind }) => kind));
    return (catalog.data.value?.agents ?? []).filter(({ kind }) => enabled.has(kind));
  });
  const agentOptions = computed(() =>
    agents.value.map(({ kind, displayName }) => {
      const readiness = catalog.data.value?.installations.find((runtime) => runtime.kind === kind);
      return {
        value: kind,
        label: displayName,
        disabled: false,
        description:
          !readiness || readiness.detected
            ? t("settings.ai.executionTargetAgentWorkflow")
            : t("settings.ai.desktopAgentNotInstalled"),
      };
    }),
  );
  const recoveryOptions = computed(() =>
    artifactState.recoveries.value.map((recovery, index) => ({
      value: recovery.sessionId,
      label: t("spaces.creation.recoveredAgentRun", {
        index: index + 1,
        count: recovery.artifacts.length,
      }),
    })),
  );
  const canRun = computed(() =>
    Boolean(
      store.activeSpaceId &&
      !artifactState.sessionId.value &&
      selectedAgent.value &&
      prompt.value.trim() &&
      agentOptions.value.some(({ value, disabled }) => value === selectedAgent.value && !disabled),
    ),
  );

  watch(open, (value) => void handleOpenChange(value));

  async function handleOpenChange(value: boolean) {
    if (!value) {
      if (running.value) await cancelActiveRun();
      return;
    }
    try {
      await aiStore.ensureLoaded();
      if (catalog.isError.value) await catalog.refetch();
      await restoreArtifacts();
      const selectionAvailable = agentOptions.value.some(
        ({ value: kind, disabled }) => kind === selectedAgent.value && !disabled,
      );
      if (!selectionAvailable) selectedAgent.value = "";
    } catch (error) {
      toast.error(String(error));
    }
  }

  async function restoreArtifacts() {
    const spaceId = store.activeSpaceId;
    if (!spaceId || artifactState.sessionId.value) return;
    const recoveries = await listRecoverableAgentCreations(spaceId);
    if (!recoveries.length) return;
    artifactState.restore(recoveries);
    status.value = t("spaces.creation.agentArtifactsReady", {
      count: recoveries[0]?.artifacts.length ?? 0,
    });
  }

  async function run(attachments: readonly string[]) {
    const spaceId = store.activeSpaceId;
    if (!spaceId || !canRun.value) return;
    const sessionId = crypto.randomUUID();
    artifactState.begin(sessionId);
    abandonRequested = false;
    running.value = true;
    status.value = t("spaces.creation.agentStarting");
    const taskId = tasks.start(
      t("spaces.creation.agentTask"),
      status.value,
      () => cancelAiJob(sessionId),
      sessionId,
    );
    try {
      const artifacts = await runAgentCreation(
        {
          kind: selectedAgent.value as AgentKind,
          prompt: prompt.value,
          attachments,
          spaceId,
          sessionId,
        },
        (event) => {
          if (event.type === "thought" || event.type === "message") status.value = event.text;
          if (event.type === "tool_started") status.value = event.title;
          tasks.detail(taskId, status.value);
        },
      );
      if (artifacts.length) artifactState.replace({ sessionId, artifacts });
      else artifactState.clear();
      tasks.succeed(taskId);
      status.value = artifacts.length
        ? t("spaces.creation.agentArtifactsReady", { count: artifacts.length })
        : t("spaces.creation.agentNoArtifacts");
    } catch (error) {
      tasks.fail(taskId, String(error));
      if (!abandonRequested) {
        toast.error(t("spaces.creation.agentFailed"), { description: String(error) });
      }
      artifactState.clear();
    } finally {
      running.value = false;
    }
  }

  async function cancelActiveRun() {
    const sessionId = artifactState.sessionId.value;
    if (!sessionId) return;
    abandonRequested = true;
    await tasks.cancel(sessionId);
  }

  function openAgentSettings() {
    open.value = false;
    void router.push({
      name: "settings",
      query: { pane: "desktopAgents", setting: "settings-desktop-agents" },
    });
  }

  onBeforeUnmount(() => {
    if (running.value) void cancelActiveRun();
  });

  return {
    agents,
    selectedAgent,
    prompt,
    running,
    status,
    agentOptions,
    recoveryOptions,
    canRun,
    openAgentSettings,
    run,
    ...artifactState,
  };
}
