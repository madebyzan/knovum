import type { ProviderDescriptor } from "../../shared/contracts/generated/ProviderDescriptor";
import type { ProviderKind } from "../../shared/contracts/generated/ProviderKind";

export interface AiProviderOption {
  readonly value: ProviderKind;
  readonly label: string;
}

export const providerOptions = (providers: readonly ProviderDescriptor[]): readonly AiProviderOption[] =>
  providers.map(({ id, displayName }) => ({ value: id, label: displayName }));
