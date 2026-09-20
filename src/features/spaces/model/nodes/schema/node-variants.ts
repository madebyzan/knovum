import { z } from "zod";

import { KNOWLEDGE_OBJECT_STATUSES } from "../../../knowledge/model/object-types";
import { spaceSubtitleCueSchema } from "../../../media/schema";
import { SPACE_CREDENTIAL_KINDS } from "../../credential";
import { baseSpaceNodeFields } from "./base-fields";
import { tiptapContentSchema } from "./content";
import { spaceResourceNodeSchemas } from "./resource-variants";
import { spaceWebsitePayloadSchema } from "./website";

const semanticNumberSchema = z
  .number()
  .finite()
  .refine(
    (value) => !Number.isInteger(value) || Number.isSafeInteger(value),
    "Semantic integers must be within the JavaScript safe integer range",
  );

export const spaceNodePayloadSchema = z.discriminatedUnion("kind", [
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("note"),
      tiptapContent: tiptapContentSchema,
      plainText: z.string().max(4 * 1024 * 1024),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("text"),
      tiptapContent: tiptapContentSchema,
      plainText: z.string().max(4 * 1024 * 1024),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("markdown"),
      text: z.string().max(4 * 1024 * 1024),
      tiptapContent: tiptapContentSchema.optional(),
      derivationState: z.enum(["pending", "failed"]).optional(),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("subtitle"),
      sourceVideoNodeId: z.string().min(1).max(128),
      sourceVideoRevision: z.string().min(1).max(128),
      language: z.string().trim().min(1).max(64),
      durationMs: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
      cues: z.array(spaceSubtitleCueSchema).min(1).max(100_000),
    })
    .strict()
    .superRefine((value, context) => {
      const invalid = value.cues.some(
        (cue, index) =>
          cue.endMs <= cue.startMs ||
          cue.endMs > value.durationMs ||
          (index > 0 && cue.startMs < value.cues[index - 1]!.startMs),
      );
      if (invalid)
        context.addIssue({
          code: "custom",
          path: ["cues"],
          message: "Subtitle cues must be ordered and bounded",
        });
    }),
  spaceWebsitePayloadSchema,
  ...spaceResourceNodeSchemas,
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("credential"),
      credentialKind: z.enum(SPACE_CREDENTIAL_KINDS),
      credentialConfigured: z.boolean().optional(),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("object"),
      objectTypeId: z.string().min(1).max(128),
      summary: z.string().max(16_000).optional(),
      status: z.enum(KNOWLEDGE_OBJECT_STATUSES),
      properties: z.record(
        z.string().min(1).max(128),
        z.union([z.string(), semanticNumberSchema, z.boolean(), z.null()]),
      ),
    })
    .strict(),
  z
    .object({
      ...baseSpaceNodeFields,
      kind: z.literal("portal"),
      targetSpaceId: z.string().min(1).max(128),
      targetSceneId: z.string().min(1).max(128).optional(),
      targetNodeId: z.string().min(1).max(128).optional(),
    })
    .strict(),
]);
