import { isString } from "es-toolkit";

import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { Space } from "../model/space";
import { requiredMap } from "./id-maps";
import { restorePackageNodes } from "./node-restore";
import {
  remapObjectTypes,
  remapWorkflowDefinition,
  restoreChunks,
  restoreDerivations,
  restoreKnowledgeEvents,
  restoreMessages,
  restoreProvenance,
  restoreRelations,
  restoreScenes,
  restoreThreads,
} from "./restore-content-records";
import type { SpacePackageRestoreContext } from "./restore-context";
import { packageSpace } from "./restore-row";
import type { SpacePackageRestoreRequest } from "./restore-transport";
import { restorePackageStructure } from "./structure-restore";

export function buildSpacePackageRestoreRequest(
  context: SpacePackageRestoreContext,
  space: Space,
): SpacePackageRestoreRequest {
  const nodes = restorePackageNodes(context);
  const nodeRevisions = new Map(
    nodes.flatMap((node) => {
      const data = node.data as SpaceNodePayload;
      return isString(data.knowledgeRevision) ? [[node.id, data.knowledgeRevision] as const] : [];
    }),
  );
  return {
    space: packageSpace(space),
    ...restorePackageStructure(context),
    objectTypes: remapObjectTypes(context.manifest.objectTypes, context.maps),
    nodes,
    scenes: restoreScenes(context),
    relations: restoreRelations(context),
    derivations: restoreDerivations(context),
    workflowDrafts: context.manifest.workflowDrafts.map((row) => ({
      ...row,
      id: requiredMap(context.maps.workflows, row.id, "Workflow"),
      definition: remapWorkflowDefinition(row.definition, context.maps),
    })),
    workflowVersions: context.manifest.workflowVersions.map((row) => ({
      ...row,
      workflowId: requiredMap(context.maps.workflows, row.workflowId, "Workflow"),
      definition: remapWorkflowDefinition(row.definition, context.maps),
    })),
    runs: context.manifest.runs.map((row) => ({
      ...row,
      id: requiredMap(context.maps.runs, row.id, "Run"),
      parentRunId: remapOptional(context.maps.runs, row.parentRunId, "parent Run"),
      retryOfRunId: remapOptional(context.maps.runs, row.retryOfRunId, "retry Run"),
      forkOfRunId: remapOptional(context.maps.runs, row.forkOfRunId, "fork Run"),
      policySnapshot: remapRunPolicy(row.policySnapshot, context.maps.runs),
    })),
    runSteps: context.manifest.runSteps.map((row) => ({
      ...row,
      id: requiredMap(context.maps.runSteps, row.id, "Run step"),
      runId: requiredMap(context.maps.runs, row.runId, "Run"),
    })),
    runBatchItems: context.manifest.runBatchItems.map((row) => ({
      ...row,
      lineageRunId: requiredMap(context.maps.runs, row.lineageRunId, "Run"),
      latestRunId: requiredMap(context.maps.runs, row.latestRunId, "Run"),
      latestStepInstanceId: requiredMap(context.maps.runSteps, row.latestStepInstanceId, "Run step"),
      childRunId: remapOptional(context.maps.runs, row.childRunId, "child Run"),
    })),
    runAttempts: context.manifest.runAttempts.map((row) => ({
      ...row,
      id: requiredMap(context.maps.runAttempts, row.id, "Run attempt"),
      runId: requiredMap(context.maps.runs, row.runId, "Run"),
      stepInstanceId: requiredMap(context.maps.runSteps, row.stepInstanceId, "Run step"),
    })),
    runEvents: context.manifest.runEvents.map((row) => ({
      ...row,
      runId: requiredMap(context.maps.runs, row.runId, "Run"),
      stepInstanceId: remapOptional(context.maps.runSteps, row.stepInstanceId, "Run step"),
      attemptId: remapOptional(context.maps.runAttempts, row.attemptId, "Run attempt"),
    })),
    automationTriggers: context.manifest.automationTriggers.map((row) => ({
      ...row,
      id: requiredMap(context.maps.automationTriggers, row.id, "Automation"),
      workflowId: requiredMap(context.maps.workflows, row.workflowId, "Workflow"),
    })),
    automationOccurrences: context.manifest.automationOccurrences.map((row) => ({
      ...row,
      id: requiredMap(context.maps.automationOccurrences, row.id, "Automation occurrence"),
      triggerId: requiredMap(context.maps.automationTriggers, row.triggerId, "Automation"),
      runId: remapOptional(context.maps.runs, row.runId, "Run"),
    })),
    externalToolBindings: context.manifest.externalToolBindings.map((row) => ({
      ...row,
      id: requiredMap(context.maps.externalTools, row.id, "external tool"),
    })),
    externalAgentConversations: context.manifest.externalAgentConversations.map((row) => ({
      ...row,
      id: requiredMap(context.maps.externalAgentConversations, row.id, "external Agent conversation"),
      runId: requiredMap(context.maps.runs, row.runId, "external Agent conversation Run"),
    })),
    externalAgentTurns: context.manifest.externalAgentTurns.map((row) => ({
      ...row,
      id: requiredMap(context.maps.externalAgentTurns, row.id, "external Agent turn"),
      conversationId: requiredMap(
        context.maps.externalAgentConversations,
        row.conversationId,
        "external Agent conversation",
      ),
    })),
    externalAgentMessages: context.manifest.externalAgentMessages.map((row) => ({
      ...row,
      id: requiredMap(context.maps.externalAgentMessages, row.id, "external Agent message"),
      conversationId: requiredMap(
        context.maps.externalAgentConversations,
        row.conversationId,
        "external Agent conversation",
      ),
      turnId: requiredMap(context.maps.externalAgentTurns, row.turnId, "external Agent turn"),
    })),
    assistantConversations: context.manifest.assistantConversations.map((row) => ({
      ...row,
      id: requiredMap(context.maps.assistantConversations, row.id, "Assistant conversation"),
    })),
    assistantMessages: context.manifest.assistantMessages.map((row) => ({
      ...row,
      id: requiredMap(context.maps.assistantMessages, row.id, "Assistant message"),
      conversationId: requiredMap(
        context.maps.assistantConversations,
        row.conversationId,
        "Assistant conversation",
      ),
      parentMessageId: remapOptional(
        context.maps.assistantMessages,
        row.parentMessageId,
        "Assistant parent message",
      ),
    })),
    provenance: restoreProvenance(context),
    knowledgeEvents: restoreKnowledgeEvents(context),
    chunks: restoreChunks(context),
    threads: restoreThreads(context),
    messages: restoreMessages(context, nodeRevisions),
  };
}

function remapOptional(map: Map<string, string>, id: string | null, kind: string): string | null {
  return id === null ? null : requiredMap(map, id, kind);
}

function remapRunPolicy(value: unknown, runs: Map<string, string>): unknown {
  if (!value || typeof value !== "object" || Array.isArray(value)) return value;
  const policy = structuredClone(value) as Record<string, unknown>;
  if (typeof policy.retryRootRunId === "string") {
    policy.retryRootRunId = requiredMap(runs, policy.retryRootRunId, "retry root Run");
  }
  return policy;
}
