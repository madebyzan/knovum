import { z } from "zod";

import { knowledgeLocatorSchema } from "../knowledge/model/schema";
import { packageTimeSchema } from "./core-schema";

const fieldSchema = z
  .object({
    key: z.string().min(1).max(128),
    label: z.string().min(1).max(128),
    kind: z.enum(["text", "number", "boolean", "date", "node-ref", "enum"]),
    required: z.boolean().optional(),
    options: z.array(z.string().trim().min(1).max(128)).min(1).max(64).optional(),
  })
  .strict();
export const objectTypeSchema = z
  .object({
    id: z.string().min(1).max(128),
    spaceId: z.string().nullable(),
    name: z.string().min(1).max(128),
    icon: z.string().min(1).max(128),
    description: z.string().max(4096),
    fields: z.array(fieldSchema).max(64),
    system: z.literal(false),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();
export const provenanceSchema = z
  .object({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    targetKind: z.enum(["node", "relation"]),
    targetId: z.string().min(1),
    sourceNodeId: z.string().min(1),
    locator: knowledgeLocatorSchema,
    sourceRevision: z.string().min(1),
    quote: z.string().max(64_000),
    status: z.enum(["active", "superseded", "invalidated"]),
    actorKind: z.enum(["human", "assistant", "agent", "system"]),
    actorId: z.string().nullable().optional(),
    createdAt: packageTimeSchema,
    updatedAt: packageTimeSchema,
  })
  .strict();
export const eventSchema = z
  .object({
    id: z.string().min(1),
    spaceId: z.string().min(1),
    entityKind: z.enum(["node", "relation", "provenance", "type"]),
    entityId: z.string().min(1),
    eventKind: z.enum([
      "created",
      "updated",
      "drafted",
      "accepted",
      "contested",
      "superseded",
      "invalidated",
      "opened",
      "resolved",
      "restored",
      "linked",
      "unlinked",
      "applied",
      "rejected",
      "moved",
    ]),
    actorKind: z.enum(["human", "assistant", "agent", "system"]),
    actorId: z.string().nullable().optional(),
    payload: z.unknown(),
    createdAt: packageTimeSchema,
  })
  .strict();
