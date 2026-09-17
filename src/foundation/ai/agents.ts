import { Channel, invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import {
  agentDescriptorSchema,
  agentInstallationSchema,
  type AgentDescriptor,
  type AgentKind,
  type AgentInstallation,
} from "./agent-models";
import { withAiError } from "./ai-runtime";

export async function loadAgentDescriptors(): Promise<AgentDescriptor[]> {
  const descriptors = await withAiError(() => invoke<unknown>("plugin:ai|list_agent_descriptors"));
  return z.array(agentDescriptorSchema).parse(descriptors);
}

export async function detectAgentInstallations(): Promise<AgentInstallation[]> {
  const runtimes = await withAiError(() => invoke<unknown>("plugin:ai|detect_agent_installations"));
  return z.array(agentInstallationSchema).parse(runtimes);
}

export interface AgentArtifact {
  name: string;
  relativePath: string;
  kind: "image" | "video" | "audio" | "document" | "file";
  size: number;
}

export interface RecoverableAgentCreation {
  sessionId: string;
  artifacts: AgentArtifact[];
}

export type AgentCreationEvent =
  | { type: "thought"; text: string }
  | { type: "message"; text: string }
  | { type: "tool_started"; id: string; title: string }
  | { type: "tool_updated"; id: string };

export function runAgentCreation(
  request: {
    kind: AgentKind;
    prompt: string;
    attachments: readonly string[];
    spaceId: string;
    sessionId: string;
  },
  onEvent: (event: AgentCreationEvent) => void,
): Promise<AgentArtifact[]> {
  const channel = new Channel<AgentCreationEvent>();
  // Tauri Channel exposes one IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  channel.onmessage = onEvent;
  return withAiError(() =>
    invoke<AgentArtifact[]>("plugin:spaces|create_agent_artifacts", { request, onEvent: channel }),
  );
}

export function discardAgentCreation(spaceId: string, sessionId: string): Promise<void> {
  return withAiError(() => invoke<void>("plugin:spaces|discard_artifact_workspace", { spaceId, sessionId }));
}

export function consumeAgentArtifact(
  spaceId: string,
  sessionId: string,
  relativePath: string,
): Promise<void> {
  return withAiError(() =>
    invoke<void>("plugin:spaces|consume_agent_artifact", { spaceId, sessionId, relativePath }),
  );
}

export function listRecoverableAgentCreations(spaceId: string): Promise<RecoverableAgentCreation[]> {
  return withAiError(() =>
    invoke<RecoverableAgentCreation[]>("plugin:spaces|list_recoverable_artifact_workspaces", { spaceId }),
  );
}

export function previewAgentArtifact(
  spaceId: string,
  sessionId: string,
  relativePath: string,
): Promise<void> {
  return withAiError(() =>
    invoke<void>("plugin:spaces|preview_generated_artifact", { spaceId, sessionId, relativePath }),
  );
}
