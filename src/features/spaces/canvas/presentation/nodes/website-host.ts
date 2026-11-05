export function normalizeHostCandidate(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/^https?:\/\//u, "")
    .replace(/^www\./u, "")
    .replace(/\/+$/u, "");
}

export function isRedundantWebsiteHost(siteName: string, host: string): boolean {
  return normalizeHostCandidate(siteName) === normalizeHostCandidate(host);
}
