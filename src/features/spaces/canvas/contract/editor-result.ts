import { z } from "zod";

import { spaceNodePayloadSchema } from "../../model/nodes/schema/node-variants";
import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES } from "../../model/relations/relation";
import { spaceEditorDocumentSchema } from "./canvas-document";

const identity = z.string().min(1).max(128);
const revision = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
const relationSchema = z
  .object({
    id: identity,
    spaceId: identity,
    sourceId: identity,
    targetId: identity,
    kind: z.enum(SPACE_RELATION_KINDS),
    rationale: z.string(),
    status: z.enum(SPACE_RELATION_STATUSES),
    statusBeforeInvalidation: z.enum(["active", "contested"]).nullable(),
    confidence: z.number().finite().min(0).max(1).nullable(),
    actorKind: z.enum(["human", "assistant", "agent", "system"]),
    actorId: identity.nullable(),
    createdAt: z.number().int(),
    updatedAt: z.number().int(),
  })
  .strict()
  .transform(({ confidence, actorId, ...relation }) => ({
    ...relation,
    ...(confidence === null ? {} : { confidence }),
    ...(actorId === null ? {} : { actorId }),
  }));
export const editorContentSchema = z
  .object({
    nodes: z.array(
      z
        .object({
          id: identity,
          initialX: z.number().finite(),
          initialY: z.number().finite(),
          data: spaceNodePayloadSchema,
        })
        .strict(),
    ),
    relations: z.array(relationSchema),
  })
  .strict();
export const spaceEditResultSchema = z
  .object({
    document: spaceEditorDocumentSchema,
    revision,
    appliedAtRevision: revision,
    mutationId: identity,
    selection: z.array(identity),
    historyReceiptId: identity.nullable(),
    content: editorContentSchema.nullable(),
  })
  .strict()
  .refine(
    (result) => result.appliedAtRevision <= result.revision,
    "A receipt cannot be ahead of the current editor state",
  );
export const editorWorkspaceSchema = z
  .object({
    document: spaceEditorDocumentSchema,
    revision,
    content: editorContentSchema,
  })
  .strict();
export type SpaceEditResult = z.infer<typeof spaceEditResultSchema>;
export type EditorWorkspace = z.infer<typeof editorWorkspaceSchema>;
export type EditorWorkspaceContent = z.infer<typeof editorContentSchema>;
