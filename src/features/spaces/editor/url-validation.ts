import { hasProtocol, withHttps } from "ufo";

export function normalizeUrl(value: string, allowedProtocols: readonly string[]): string | undefined {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const candidate = hasProtocol(trimmed) ? trimmed : withHttps(trimmed);
  try {
    const protocol = new URL(candidate).protocol;
    return allowedProtocols.includes(protocol) ? candidate : undefined;
  } catch {
    return undefined;
  }
}
