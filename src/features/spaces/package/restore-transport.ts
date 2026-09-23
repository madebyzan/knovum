import { invoke } from "@tauri-apps/api/core";

import type { WorkflowDefinition } from "../contracts/generated/WorkflowDefinition";
import type { GroundedCitation } from "../knowledge/model/grounded-citation";
import type { GroundedMessage } from "../knowledge/model/grounded-message";
import type { GroundedScope } from "../knowledge/model/grounded-scope";
import type { KnowledgeChunkOrigin } from "../knowledge/model/knowledge-chunk";
import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import type { SpaceKnowledgeEvent, SpaceProvenance } from "../knowledge/model/provenance";
import type { KnowledgeChunkKind, KnowledgeLocator } from "../knowledge/model/schema";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelationKind, SpaceRelationStatus } from "../model/relations/relation";
import type { Space } from "../model/space";
import type { StructureLayout, StructureNode } from "../model/structure/structure-contract";
import type { SpaceScene } from "../scenes/scene";
import type { SpacePackageManifest } from "./manifest-schema";
import type { PackageRelationStyle } from "./structure-schema";

export interface SpacePackageRestoreRequest {
  space: Pick<Space, "id" | "kind" | "title" | "lastOpenedAt" | "createdAt" | "updatedAt">;
  structureNodes: readonly StructureNode[];
  layouts: readonly StructureLayout[];
  relationStyles: readonly PackageRelationStyle[];
  objectTypes: readonly Omit<SpaceKnowledgeObjectType, "spaceId" | "system">[];
  nodes: readonly {
    id: string;
    kind: string;
    data: SpaceNodePayload;
    createdAt: number;
    updatedAt: number;
  }[];
  scenes: readonly Omit<SpaceScene, "spaceId">[];
  relations: readonly {
    id: string;
    sourceId: string;
    targetId: string;
    kind: SpaceRelationKind;
    rationale: string;
    status: SpaceRelationStatus;
    statusBeforeInvalidation: "active" | "contested" | null;
    confidence: number | null;
    actorKind: "human" | "assistant" | "agent" | "system";
    actorId: string | null;
    createdAt: number;
    updatedAt: number;
  }[];
  derivations: readonly {
    id: string;
    outputNodeId: string;
    inputs: readonly {
      nodeId: string | null;
      sourceIdentity: string;
      sourceSnapshot: { title: string; kind: string; originalUrl: string | null; platform: string | null };
      role: "primary" | "source" | "context" | "style-reference";
      sourceRevision: string;
      revisionBasis: "asset" | "rendered" | "knowledge";
      inputRevision: string;
      inputSnapshot: null | Record<string, unknown>;
    }[];
    actionKind:
      | "remove-image-background"
      | "split-image-layers"
      | "upscale-image"
      | "generate-image-angle"
      | "generate-visualization"
      | "extract-subtitles"
      | "render-subtitled-video"
      | "render-video-edit"
      | "draft-document"
      | "export-document-as-docx"
      | "export-document-as-pdf"
      | "render-presentation"
      | "generate-video";
    actionVersion: 1;
    engine: string;
    status: "completed" | "failed" | "cancelled";
    error: string;
    createdAt: number;
    updatedAt: number;
    completedAt: number;
    groundedDocument?: {
      citations: readonly unknown[];
      provider: string;
      model: string;
    };
    generatedImage?: {
      intent: string;
      style: string;
      aspectRatio: string;
      provider: string;
      model: string;
      revisedPrompt: string | null;
    };
    generatedVideo?: {
      prompt: string;
      aspectRatio: "1:1" | "16:9" | "9:16";
      durationSeconds: number;
      provider: string;
      model: string;
      operationId: string;
    };
  }[];
  workflowDrafts: readonly {
    id: string;
    revision: number;
    definition: WorkflowDefinition;
    createdAt: number;
    updatedAt: number;
  }[];
  workflowVersions: readonly {
    workflowId: string;
    version: number;
    definition: WorkflowDefinition;
    publishedAt: number;
  }[];
  runs: SpacePackageManifest["runs"];
  runSteps: SpacePackageManifest["runSteps"];
  runBatchItems: SpacePackageManifest["runBatchItems"];
  runAttempts: SpacePackageManifest["runAttempts"];
  runEvents: SpacePackageManifest["runEvents"];
  automationTriggers: SpacePackageManifest["automationTriggers"];
  automationOccurrences: SpacePackageManifest["automationOccurrences"];
  externalToolBindings: SpacePackageManifest["externalToolBindings"];
  externalAgentConversations: SpacePackageManifest["externalAgentConversations"];
  externalAgentTurns: SpacePackageManifest["externalAgentTurns"];
  externalAgentMessages: SpacePackageManifest["externalAgentMessages"];
  assistantConversations: SpacePackageManifest["assistantConversations"];
  assistantMessages: SpacePackageManifest["assistantMessages"];
  provenance: readonly Omit<SpaceProvenance, "spaceId">[];
  knowledgeEvents: readonly Omit<SpaceKnowledgeEvent, "spaceId">[];
  chunks: readonly {
    id: string;
    nodeId: string;
    origin: KnowledgeChunkOrigin;
    kind: KnowledgeChunkKind;
    sourceRevision: string;
    text: string;
    locator: KnowledgeLocator;
    metadata: Record<string, string | number | boolean | null>;
    createdAt: number;
    updatedAt: number;
  }[];
  threads: readonly {
    id: string;
    scope: GroundedScope;
    scopeNodeIds: readonly string[];
    title: string;
    createdAt: number;
    updatedAt: number;
  }[];
  messages: readonly {
    id: string;
    threadId: string;
    role: GroundedMessage["role"];
    content: string;
    citations: readonly GroundedCitation[];
    provider?: string;
    model?: string;
    createdAt: number;
  }[];
}

export function restoreSpacePackageData(request: SpacePackageRestoreRequest): Promise<void> {
  return invoke("plugin:spaces|restore_space_package", { request });
}
