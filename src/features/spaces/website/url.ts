import { hasProtocol, parseURL, withHttps } from "ufo";

export function normalizeSpaceUrl(value: string): string | undefined {
  const input = value.trim();
  if (!input || input.length > 4096 || /\s/u.test(input)) return undefined;
  const candidate = hasProtocol(input) ? input : input.includes(".") ? withHttps(input) : undefined;
  if (!candidate) return undefined;
  try {
    const url = new URL(candidate);
    return url.protocol === "http:" || url.protocol === "https:" ? url.href : undefined;
  } catch {
    return undefined;
  }
}

export function websiteDefaultTitle(url: string): string {
  const parsed = parseURL(url);
  return (parsed.host || "").replace(/^www\./u, "");
}
