import { isPlainObject, isString } from "es-toolkit";

import { groundedCitationSchema } from "../knowledge/model/grounded-citation";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { Space } from "../model/space";
import { remapGroundedCitation } from "./content-remap";
import { requiredMap } from "./id-maps";
import { restorePackageNodes } from "./node-restore";
import {
  remapObjectTypes,
  restoreChunks,
  restoreDerivations,
  restoreKnowledgeEvents,
  restoreProvenance,
  restoreRelations,
  restoreScenes,
} from "./restore-content";
import type { SpacePackageRestoreContext } from "./restore-context";
import { prepareSpaceRestore } from "./restore-space";
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
    space: prepareSpaceRestore(space),
    ...restorePackageStructure(context),
    objectTypes: remapObjectTypes(context.manifest.objectTypes, context.maps),
    nodes,
    scenes: restoreScenes(context),
    relations: restoreRelations(context),
    derivations: restoreDerivations(context),
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
    externalAgentConversations: context.manifest.externalAgentConversations.map((row) => ({
      ...row,
      id: requiredMap(context.maps.externalAgentConversations, row.id, "external Agent conversation"),
      runId: requiredMap(context.maps.runs, row.runId, "external Agent conversation Run"),
      assistantConversationId: requiredMap(
        context.maps.assistantConversations,
        row.assistantConversationId,
        "Assistant conversation",
      ),
    })),
    externalAgentTurns: context.manifest.externalAgentTurns.map((row) => ({
      ...row,
      id: requiredMap(context.maps.externalAgentTurns, row.id, "external Agent turn"),
      assistantMessageId: row.assistantMessageId
        ? requiredMap(context.maps.assistantMessages, row.assistantMessageId, "Assistant message")
        : null,
      conversationId: requiredMap(
        context.maps.externalAgentConversations,
        row.conversationId,
        "external Agent conversation",
      ),
    })),
    assistantConversations: context.manifest.assistantConversations.map((row) => ({
      ...row,
      id: requiredMap(context.maps.assistantConversations, row.id, "Assistant conversation"),
    })),
    assistantMessages: context.manifest.assistantMessages.map((row) => ({
      ...row,
      id: requiredMap(context.maps.assistantMessages, row.id, "Assistant message"),
      content: {
        ...row.content,
        ...(row.content.citations
          ? {
              citations: row.content.citations.map((citation) =>
                remapGroundedCitation(groundedCitationSchema.parse(citation), context.maps, nodeRevisions),
              ),
            }
          : {}),
      },
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
  };
}

function remapOptional(map: Map<string, string>, id: string | null, kind: string): string | null {
  return id === null ? null : requiredMap(map, id, kind);
}

function remapRunPolicy(value: unknown, runs: Map<string, string>): unknown {
  if (!isPlainObject(value)) return value;
  const policy: Record<string, unknown> = structuredClone(value);
  if (typeof policy.retryRootRunId === "string") {
    policy.retryRootRunId = requiredMap(runs, policy.retryRootRunId, "retry root Run");
  }
  return policy;
}
