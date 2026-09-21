import { isPlainObject } from "es-toolkit";
import { z } from "zod";

import type { TiptapJsonNode } from "../../../editor/tiptap-document";

const MAX_JSON_DEPTH = 64;
const MAX_JSON_CONTAINER_ENTRIES = 10_000;
const MAX_JSON_VALUES = 100_000;
const MAX_TIPTAP_DEPTH = 64;
const MAX_TIPTAP_NODES = 100_000;
const jsonPrimitiveSchema = z.union([z.string(), z.number().finite(), z.boolean(), z.null()]);

function jsonValueSchemaAt(depth: number): z.ZodType<unknown> {
  return z.lazy(() => {
    if (depth >= MAX_JSON_DEPTH) return jsonPrimitiveSchema;
    const child = jsonValueSchemaAt(depth + 1);
    return z.union([
      jsonPrimitiveSchema,
      z.array(child).max(MAX_JSON_CONTAINER_ENTRIES),
      z
        .record(z.string().max(128), child)
        .refine(
          (value) => Object.keys(value).length <= MAX_JSON_CONTAINER_ENTRIES,
          "JSON object has too many fields",
        ),
    ]);
  });
}

function countJsonValues(value: unknown, remaining = MAX_JSON_VALUES + 1): number {
  if (remaining <= 0) return MAX_JSON_VALUES + 1;
  if (!isPlainObject(value) && !Array.isArray(value)) return 1;
  const children = Array.isArray(value) ? value : Object.values(value);
  let count = 1;
  for (const child of children) {
    count += countJsonValues(child, remaining - count);
    if (count > MAX_JSON_VALUES) return count;
  }
  return count;
}

const jsonAttrsSchema = z
  .record(z.string().max(128), jsonValueSchemaAt(1))
  .refine(
    (value) => Object.keys(value).length <= MAX_JSON_CONTAINER_ENTRIES,
    "JSON object has too many fields",
  )
  .refine((value) => countJsonValues(value) <= MAX_JSON_VALUES, "JSON object contains too many values");
const tiptapMarkSchema = z
  .object({
    type: z.string().min(1).max(64),
    attrs: jsonAttrsSchema.optional(),
  })
  .strict();

function tiptapNodeSchemaAt(depth: number): z.ZodType<unknown> {
  return z.lazy(() =>
    z
      .object({
        type: z.string().min(1).max(64),
        attrs: jsonAttrsSchema.optional(),
        content:
          depth >= MAX_TIPTAP_DEPTH
            ? z.array(z.never()).max(0).optional()
            : z
                .array(tiptapNodeSchemaAt(depth + 1))
                .max(MAX_TIPTAP_NODES)
                .optional(),
        marks: z.array(tiptapMarkSchema).max(256).optional(),
        text: z
          .string()
          .max(4 * 1024 * 1024)
          .optional(),
      })
      .strict(),
  );
}

function isTiptapRecord(value: unknown): value is { content?: unknown } {
  return isPlainObject(value);
}

function countTiptapNodes(value: unknown): number {
  if (!isTiptapRecord(value)) return 0;
  let count = 0;
  const stack: unknown[] = [value];
  while (stack.length) {
    const node = stack.pop();
    if (!isTiptapRecord(node)) continue;
    count += 1;
    if (count > MAX_TIPTAP_NODES) return count;
    if (Array.isArray(node.content)) stack.push(...node.content);
  }
  return count;
}

export const tiptapContentSchema = tiptapNodeSchemaAt(0)
  .refine(
    (value) => isPlainObject(value) && (value as { type?: unknown }).type === "doc",
    "Tiptap root must be doc",
  )
  .refine((value) => countTiptapNodes(value) <= MAX_TIPTAP_NODES, "Tiptap content contains too many nodes")
  .transform((value) => value as TiptapJsonNode);

export const httpUrlSchema = z
  .string()
  .max(4096)
  .url()
  .refine((value) => {
    try {
      return ["http:", "https:"].includes(new URL(value).protocol);
    } catch {
      return false;
    }
  }, "URL must use HTTP or HTTPS");
