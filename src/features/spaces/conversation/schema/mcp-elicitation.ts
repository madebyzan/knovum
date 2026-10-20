import { z } from "zod";

const common = { title: z.string().optional(), description: z.string().optional() };
const titled = z.object({ const: z.string(), title: z.string() });
const stringField = z.object({
  ...common,
  type: z.literal("string"),
  enum: z.array(z.string()).optional(),
  enumNames: z.array(z.string()).optional(),
  oneOf: z.array(titled).optional(),
  default: z.string().optional(),
  format: z.enum(["email", "uri", "date", "date-time"]).optional(),
  minLength: z.number().optional(),
  maxLength: z.number().optional(),
});
const numeric = {
  ...common,
  enum: z.array(z.number()).optional(),
  default: z.number().optional(),
  minimum: z.number().optional(),
  maximum: z.number().optional(),
};
const arrayField = z.object({
  ...common,
  type: z.literal("array"),
  default: z.array(z.string()).optional(),
  minItems: z.number().optional(),
  maxItems: z.number().optional(),
  items: z.union([
    z.object({ type: z.literal("string"), enum: z.array(z.string()) }),
    z.object({ anyOf: z.array(titled) }),
  ]),
});
const elicitationField = z.union([
  stringField,
  z.object({ ...numeric, type: z.literal("number") }),
  z.object({ ...numeric, type: z.literal("integer") }),
  z.object({ ...common, type: z.literal("boolean"), default: z.boolean().optional() }),
  arrayField,
]);
export const mcpElicitationSchema = z.discriminatedUnion("mode", [
  z.object({
    mode: z.literal("form"),
    message: z.string(),
    requestedSchema: z.object({
      type: z.literal("object"),
      properties: z.record(z.string(), elicitationField),
      required: z.array(z.string()).optional(),
    }),
  }),
  z.object({
    mode: z.literal("url"),
    message: z.string(),
    url: z
      .string()
      .url()
      .refine((value) => ["https:", "http:"].includes(new URL(value).protocol)),
    elicitationId: z.string(),
  }),
]);
export type McpElicitationField = z.infer<typeof elicitationField>;
