import { z } from "zod";

import { objectFieldSchema } from "../knowledge/model/object-field-schema";
import { knowledgeLocatorSchema } from "../knowledge/model/schema";
import { packageTimeSchema } from "./core-schema";

export const objectTypeSchema = z
  .object({
    id: z.string().min(1).max(128),
    spaceId: z.string().nullable(),
    name: z.string().min(1).max(128),
    icon: z.string().min(1).max(128),
    description: z.string().max(4096),
    fields: z.array(objectFieldSchema).max(64),
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
