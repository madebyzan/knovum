import { normalizeSpaceUrl } from "../website/url";

export interface SpaceUrlSource {
  kind: "url";
  url: string;
}

export function classifySpaceUrl(value: string): SpaceUrlSource | undefined {
  const normalized = normalizeSpaceUrl(value);
  return normalized ? { kind: "url", url: normalized } : undefined;
}
