import { z } from "zod";

import type { ProviderDescriptor } from "../../shared/contracts/generated/ProviderDescriptor";
import { invokeCommand } from "../ipc/invoke-command";
import { aiCapabilitySchema, aiProviderSchema } from "./schema/models";

const providerDescriptorSchema: z.ZodType<ProviderDescriptor> = z
  .object({
    id: aiProviderSchema,
    displayName: z.string().min(1),
    defaultBaseUrl: z.union([z.literal(""), z.url()]),
    capabilities: z.array(aiCapabilitySchema).min(1),
  })
  .strict();

export type { ProviderDescriptor } from "../../shared/contracts/generated/ProviderDescriptor";

export async function loadProviderDescriptors(): Promise<ProviderDescriptor[]> {
  return z
    .array(providerDescriptorSchema)
    .parse(await invokeCommand<unknown>("plugin:ai|list_provider_descriptors"));
}

export function requireProvider(
  providers: readonly ProviderDescriptor[],
  id: ProviderDescriptor["id"],
): ProviderDescriptor {
  const provider = providers.find((candidate) => candidate.id === id);
  if (!provider) throw new Error(`Provider is not registered: ${id}`);
  return provider;
}
