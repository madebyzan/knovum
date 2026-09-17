import { z } from "zod";

export const spaceCredentialSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("api-key"), secret: z.string().max(64 * 1024) }),
  z.object({
    kind: z.literal("account-password"),
    username: z.string().max(8 * 1024),
    password: z.string().max(64 * 1024),
  }),
]);

export type SpaceCredential = z.infer<typeof spaceCredentialSchema>;
export type SpaceCredentialKind = SpaceCredential["kind"];
export const SPACE_CREDENTIAL_KINDS = spaceCredentialSchema.options.map(
  (option) => option.shape.kind.value,
) as [SpaceCredentialKind, ...SpaceCredentialKind[]];
