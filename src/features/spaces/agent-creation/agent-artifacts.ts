import { Channel } from "@tauri-apps/api/core";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { AgentArtifact } from "../../../shared/contracts/generated/AgentArtifact";
import type { AgentCreationEvent } from "../contracts/generated/AgentCreationEvent";
import type { AgentCreationRequest } from "../contracts/generated/AgentCreationRequest";
import type { RecoverableAgentCreation } from "../contracts/generated/RecoverableAgentCreation";
export type { AgentArtifact } from "../../../shared/contracts/generated/AgentArtifact";
export type { RecoverableAgentCreation } from "../contracts/generated/RecoverableAgentCreation";

export function runAgentCreation(
  request: AgentCreationRequest,
  onEvent: (event: AgentCreationEvent) => void,
): Promise<AgentArtifact[]> {
  const channel = new Channel<AgentCreationEvent>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onEvent;
  return invokeCommand<AgentArtifact[]>("plugin:spaces|create_agent_artifacts", {
    request,
    onEvent: channel,
  });
}

export function discardAgentCreation(spaceId: string, sessionId: string): Promise<void> {
  return invokeCommand<void>("plugin:spaces|discard_artifact_workspace", { spaceId, sessionId });
}

export function consumeAgentArtifact(
  spaceId: string,
  sessionId: string,
  relativePath: string,
): Promise<void> {
  return invokeCommand<void>("plugin:spaces|consume_agent_artifact", { spaceId, sessionId, relativePath });
}

export function listRecoverableAgentCreations(spaceId: string): Promise<RecoverableAgentCreation[]> {
  return invokeCommand<RecoverableAgentCreation[]>("plugin:spaces|list_recoverable_artifact_workspaces", {
    spaceId,
  });
}

export function previewAgentArtifact(
  spaceId: string,
  sessionId: string,
  relativePath: string,
): Promise<void> {
  return invokeCommand<void>("plugin:spaces|preview_generated_artifact", {
    spaceId,
    sessionId,
    relativePath,
  });
}
