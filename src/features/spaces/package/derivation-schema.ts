import { z } from "zod";

import { SPACE_NODE_KINDS } from "../model/nodes/node-kind";
import { packageTimeSchema } from "./core-schema";

export const derivationSchema = z
  .object({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    outputNodeId: z.string().min(1),
    inputs: z
      .array(
        z
          .object({
            nodeId: z.string().min(1).nullable(),
            sourceIdentity: z.string().min(1),
            sourceSnapshot: z
              .object({
                title: z.string(),
                kind: z.enum(SPACE_NODE_KINDS),
                originalUrl: z.string().nullable(),
                platform: z.string().nullable(),
              })
              .strict(),
            role: z.enum(["primary", "source", "context", "style-reference"]),
            sourceRevision: z.string().min(1),
          })
          .strict(),
      )
      .min(1)
      .max(1_000),
    actionKind: z.enum([
      "remove-image-background",
      "generate-visualization",
      "extract-subtitles",
      "render-subtitled-video",
      "draft-document",
      "export-document-as-docx",
      "export-document-as-pdf",
      "render-presentation",
      "generate-video",
    ]),
    actionVersion: z.literal(1),
    engine: z.string().min(1).max(256),
    status: z.enum(["completed", "failed"]),
    error: z.string().max(16_000),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
    completedAt: packageTimeSchema,
    groundedDocument: z
      .object({
        citations: z.array(z.unknown()).max(10_000),
        provider: z.string().min(1).max(128),
        model: z.string().min(1).max(256),
      })
      .strict()
      .optional(),
    generatedImage: z
      .object({
        intent: z.string().min(1).max(4_000),
        style: z.string().max(2_000),
        aspectRatio: z.string().min(1).max(32),
        provider: z.string().min(1).max(128),
        model: z.string().min(1).max(256),
        revisedPrompt: z.string().nullable(),
      })
      .strict()
      .optional(),
    generatedVideo: z
      .object({
        prompt: z.string().min(1),
        aspectRatio: z.enum(["1:1", "16:9", "9:16"]),
        durationSeconds: z.number().int().min(5).max(15),
        provider: z.string().min(1).max(128),
        model: z.string().min(1).max(256),
        operationId: z.string().min(1).max(512),
      })
      .strict()
      .optional(),
  })
  .strict()
  .superRefine((value, context) => {
    const lifecycleValid = value.status === "completed" ? value.error === "" : value.error.trim().length > 0;
    if (!lifecycleValid)
      context.addIssue({ code: "custom", message: "Derivation lifecycle state is invalid" });
    validateActionMetadata(
      {
        action: value.actionKind,
        status: value.status,
        metadata: value.groundedDocument,
        expectedAction: "draft-document",
        label: "Document",
      },
      context,
    );
    validateActionMetadata(
      {
        action: value.actionKind,
        status: value.status,
        metadata: value.generatedImage,
        expectedAction: "generate-visualization",
        label: "Generated image",
      },
      context,
    );
    validateActionMetadata(
      {
        action: value.actionKind,
        status: value.status,
        metadata: value.generatedVideo,
        expectedAction: "generate-video",
        label: "Generated video",
      },
      context,
    );
    const primary = value.inputs.filter(({ role }) => role === "primary");
    if (primary.length !== 1 || value.inputs[0]?.role !== "primary") {
      context.addIssue({
        code: "custom",
        message: "Derivation requires one leading primary input",
        path: ["inputs"],
      });
    }
  });

type RefinementContext = Parameters<Parameters<typeof derivationSchema.superRefine>[0]>[1];

function validateActionMetadata(
  value: { action: string; status: string; metadata: unknown; expectedAction: string; label: string },
  context: RefinementContext,
): void {
  const { action, status, metadata, expectedAction, label } = value;
  const valid =
    action === expectedAction ? status === "failed" || metadata !== undefined : metadata === undefined;
  if (!valid) context.addIssue({ code: "custom", message: `${label} metadata is invalid` });
}
