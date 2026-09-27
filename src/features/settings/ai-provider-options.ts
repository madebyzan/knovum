import type { ProviderDescriptor } from "../../foundation/ai/providers";
import type { AiProvider } from "../../foundation/ai/schema/models";

export interface AiProviderOption {
  readonly value: AiProvider;
  readonly label: string;
}

export const providerOptions = (providers: readonly ProviderDescriptor[]): readonly AiProviderOption[] =>
  providers.map(({ id, displayName }) => ({ value: id, label: displayName }));
