import { computed, onBeforeUnmount, ref, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import type { AgentKind } from "../../../foundation/ai/agent-models";
import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useSpaceStore } from "../session/space-store";
import {
  decideAgentPermission,
  listRecoverableAgentCreations,
  runAgentCreation,
  type AgentPermissionOption,
} from "./agent-artifacts";
import { useAgentArtifacts } from "./use-agent-artifacts";
import { useAgentCreationCatalog } from "./use-agent-creation-catalog";

export function useAgentCreation(open: Ref<boolean>) {
  const { t } = useI18n();
  const router = useRouter();
  const store = useSpaceStore();
  const tasks = useTaskStore();
  const aiStore = useAiSettingsStore();
  const catalog = useAgentCreationCatalog();
  const artifactState = useAgentArtifacts(open);
  const selectedAgent = ref("");
  const prompt = ref("");
  const running = ref(false);
  const status = ref("");
  const pendingPermission = ref<{
    requestId: string;
    title: string;
    options: AgentPermissionOption[];
  } | null>(null);
  let abandonRequested = false;

  const { agents, agentOptions } = catalog;
  const recoveryOptions = computed(() =>
    artifactState.recoveries.value.map((recovery, index) => ({
      value: recovery.sessionId,
      label: t("spaces.creation.agent.recovered.run", {
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
    status.value = t("spaces.creation.agent.artifacts.ready", {
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
    status.value = t("spaces.creation.agent.starting");
    const taskId = tasks.start(
      t("spaces.creation.agent.task"),
      status.value,
      () => cancelAiJob(sessionId),
      sessionId,
    );
    try {
      const artifacts = await runAgentCreation(
        {
          kind: selectedAgent.value as AgentKind,
          prompt: prompt.value,
          attachments: [...attachments],
          spaceId,
          sessionId,
        },
        (event) => {
          if (event.type === "thought" || event.type === "message") status.value = event.text;
          if (event.type === "tool_started") status.value = event.title;
          if (event.type === "permission_requested") {
            pendingPermission.value = event;
            status.value = event.title;
          }
          tasks.detail(taskId, status.value);
        },
      );
      if (artifacts.length) artifactState.replace({ sessionId, artifacts });
      else artifactState.clear();
      tasks.succeed(taskId);
      status.value = artifacts.length
        ? t("spaces.creation.agent.artifacts.ready", { count: artifacts.length })
        : t("spaces.creation.agent.artifacts.none");
    } catch (error) {
      tasks.fail(taskId, String(error));
      if (!abandonRequested) {
        toast.error(t("spaces.creation.agent.failed"), { description: String(error) });
      }
      artifactState.clear();
    } finally {
      pendingPermission.value = null;
      running.value = false;
    }
  }

  async function decidePermission(optionId: string | null) {
    const permission = pendingPermission.value;
    const activeSessionId = artifactState.sessionId.value;
    if (!permission || !activeSessionId) return;
    pendingPermission.value = null;
    await decideAgentPermission(activeSessionId, permission.requestId, optionId);
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
    pendingPermission,
    agentOptions,
    recoveryOptions,
    canRun,
    openAgentSettings,
    run,
    decidePermission,
    ...artifactState,
  };
}
