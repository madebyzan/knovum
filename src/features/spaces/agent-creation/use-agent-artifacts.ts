import { computed, reactive, ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { AgentArtifact } from "../../../shared/contracts/generated/AgentArtifact";
import { prepareAgentArtifactImport } from "../assets/asset-operations";
import { spaceImportContext } from "../assets/import-context";
import { importSpaceSource } from "../assets/import-session";
import { useNodeOperationGate } from "../canvas/use-node-operation-gate";
import type { RecoverableAgentCreation } from "../contracts/generated/RecoverableAgentCreation";
import { useSpaceStore } from "../session/space-store";
import { consumeAgentArtifact, discardAgentCreation, previewAgentArtifact } from "./agent-artifacts";

export function useAgentArtifacts(open: Ref<boolean>) {
  const { t } = useI18n();
  const store = useSpaceStore();
  const operations = useNodeOperationGate();
  const sessionId = ref<string>();
  const artifacts = ref<AgentArtifact[]>([]);
  const recoveries = ref<RecoverableAgentCreation[]>([]);
  const selectedArtifacts = reactive(new Set<string>());

  function replace(session: RecoverableAgentCreation) {
    recoveries.value = [session, ...recoveries.value.filter(({ sessionId: id }) => id !== session.sessionId)];
    sessionId.value = session.sessionId;
    artifacts.value = session.artifacts;
    selectedArtifacts.clear();
    for (const artifact of session.artifacts) selectedArtifacts.add(artifact.relativePath);
  }

  function restore(sessions: RecoverableAgentCreation[]) {
    recoveries.value = sessions;
    const first = sessions[0];
    if (first) replace(first);
  }

  function selectRecovery(targetSessionId: string) {
    const session = recoveries.value.find(({ sessionId: id }) => id === targetSessionId);
    if (session) replace(session);
  }

  function begin(nextSessionId: string) {
    sessionId.value = nextSessionId;
    artifacts.value = [];
    selectedArtifacts.clear();
  }

  function removeConsumed(relativePath: string) {
    artifacts.value = artifacts.value.filter((artifact) => artifact.relativePath !== relativePath);
    selectedArtifacts.delete(relativePath);
    recoveries.value = recoveries.value.map((recovery) =>
      recovery.sessionId === sessionId.value
        ? {
            ...recovery,
            artifacts: recovery.artifacts.filter((artifact) => artifact.relativePath !== relativePath),
          }
        : recovery,
    );
  }

  function setArtifactSelected(relativePath: string, selected: boolean) {
    if (selected) selectedArtifacts.add(relativePath);
    else selectedArtifacts.delete(relativePath);
  }

  async function discard(spaceId: string, targetSessionId = sessionId.value) {
    if (!targetSessionId) return;
    await discardAgentCreation(spaceId, targetSessionId);
    recoveries.value = recoveries.value.filter(({ sessionId: id }) => id !== targetSessionId);
    if (sessionId.value === targetSessionId) {
      clear();
      const next = recoveries.value[0];
      if (next) replace(next);
    }
  }

  async function discardCurrent() {
    if (!store.activeSpaceId) return;
    try {
      await discard(store.activeSpaceId);
    } catch (error) {
      toast.error(t("spaces.creation.agent.artifacts.discardFailed"), {
        description: getErrorMessage(error),
      });
    }
  }

  async function importSelected() {
    const spaceId = store.activeSpaceId;
    const currentSessionId = sessionId.value;
    if (!spaceId || !currentSessionId || !operations.allows("create")) return;
    try {
      for (const artifact of artifacts.value.filter(({ relativePath }) =>
        selectedArtifacts.has(relativePath),
      )) {
        if (!operations.allows("create")) return;
        // oxlint-disable-next-line eslint/no-await-in-loop -- Managed Space imports are intentionally serialized.
        const imported = await importSpaceSource({
          context: spaceImportContext(store),
          spaceId,
          source: {
            prepare: () => prepareAgentArtifactImport(spaceId, currentSessionId, artifact.relativePath),
            finalize: () => consumeAgentArtifact(spaceId, currentSessionId, artifact.relativePath),
          },
          position: store.nextInsertionPoint(),
          labels: {
            title: t("spaces.importExport.progress.resource"),
            preparing: t("spaces.importExport.progress.preparing"),
          },
        });
        if (!imported.length) return;
        removeConsumed(artifact.relativePath);
      }
    } catch (error) {
      toast.error(t("spaces.importExport.import.failed"), { description: getErrorMessage(error) });
      return;
    }
    open.value = false;
    try {
      await discard(spaceId, currentSessionId);
    } catch (error) {
      toast.error(t("spaces.creation.agent.artifacts.discardFailed"), {
        description: getErrorMessage(error),
      });
    }
  }

  function preview(artifact: AgentArtifact) {
    if (!store.activeSpaceId || !sessionId.value) return;
    void previewAgentArtifact(store.activeSpaceId, sessionId.value, artifact.relativePath).catch((error) =>
      toast.error(getErrorMessage(error)),
    );
  }

  function clear() {
    sessionId.value = undefined;
    artifacts.value = [];
    selectedArtifacts.clear();
  }

  return {
    sessionId,
    artifacts,
    recoveries,
    selectedArtifacts,
    setArtifactSelected,
    selectedCount: computed(() => selectedArtifacts.size),
    replace,
    restore,
    selectRecovery,
    begin,
    discard,
    discardCurrent,
    importSelected,
    preview,
    clear,
  };
}
