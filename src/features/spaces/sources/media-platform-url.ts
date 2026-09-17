interface MediaPlatformRule {
  domains: readonly string[];
  isContentUrl(url: URL): boolean;
}

const MEDIA_PLATFORM_RULES: readonly MediaPlatformRule[] = [
  {
    domains: ["youtube.com"],
    isContentUrl: ({ pathname, searchParams }) =>
      (pathname === "/watch" && searchParams.has("v")) || /^\/(shorts|live|embed)\/[^/]+/.test(pathname),
  },
  { domains: ["youtu.be"], isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
  {
    domains: ["bilibili.com"],
    isContentUrl: ({ pathname }) => /^\/(video|bangumi\/play|audio)\/[^/]+/.test(pathname),
  },
  { domains: ["b23.tv"], isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
  {
    domains: ["douyin.com"],
    isContentUrl: ({ pathname }) => /^\/(video|share\/video)\/[^/]+/.test(pathname),
  },
  { domains: ["v.douyin.com"], isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
  {
    domains: ["tiktok.com"],
    isContentUrl: ({ pathname }) => /^\/@[^/]+\/video\/[^/]+/.test(pathname),
  },
  { domains: ["vm.tiktok.com", "vt.tiktok.com"], isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
  {
    domains: ["instagram.com"],
    isContentUrl: ({ pathname }) => /^\/(p|reel|tv)\/[^/]+/.test(pathname),
  },
  { domains: ["vimeo.com"], isContentUrl: ({ pathname }) => /^\/\d+(?:\/|$)/.test(pathname) },
  {
    domains: ["player.vimeo.com"],
    isContentUrl: ({ pathname }) => /^\/video\/\d+(?:\/|$)/.test(pathname),
  },
  {
    domains: ["x.com", "twitter.com"],
    isContentUrl: ({ pathname }) => /^\/(?:[^/]+|i)\/status\/\d+/.test(pathname),
  },
  {
    domains: ["facebook.com"],
    isContentUrl: ({ pathname, searchParams }) =>
      (pathname === "/watch" && searchParams.has("v")) ||
      /^\/(reel\/[^/]+|[^/]+\/videos\/[^/]+)/.test(pathname),
  },
  { domains: ["fb.watch"], isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
];

export function isMediaPlatformContentUrl(url: URL): boolean {
  return MEDIA_PLATFORM_RULES.some(
    ({ domains, isContentUrl }) =>
      domains.some((domain) => url.hostname === domain || url.hostname.endsWith(`.${domain}`)) &&
      isContentUrl(url),
  );
}
