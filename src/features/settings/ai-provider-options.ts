import type { AiProvider } from "../../foundation/ai/ai-models";
import type { ProviderDescriptor } from "../../foundation/ai/providers";

export interface AiProviderOption {
  readonly value: AiProvider;
  readonly label: string;
}

export const providerOptions = (providers: readonly ProviderDescriptor[]): readonly AiProviderOption[] =>
  providers.map(({ id, displayName }) => ({ value: id, label: displayName }));
