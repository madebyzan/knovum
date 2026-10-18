interface MediaPlatformRule {
  domains: readonly string[];
  /** What the platform serves at these addresses, which is what the canvas reserves for them. */
  serves: "audio" | "video";
  isContentUrl(url: URL): boolean;
}

const MEDIA_PLATFORM_RULES: readonly MediaPlatformRule[] = [
  {
    domains: ["youtube.com"],
    serves: "video",
    isContentUrl: ({ pathname, searchParams }) =>
      (pathname === "/watch" && searchParams.has("v")) || /^\/(shorts|live|embed)\/[^/]+/.test(pathname),
  },
  { domains: ["youtu.be"], serves: "video", isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
  {
    domains: ["bilibili.com"],
    serves: "audio",
    isContentUrl: ({ pathname }) => /^\/audio\/[^/]+/.test(pathname),
  },
  {
    domains: ["bilibili.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/(video|bangumi\/play)\/[^/]+/.test(pathname),
  },
  { domains: ["b23.tv"], serves: "video", isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
  {
    domains: ["douyin.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/(video|share\/video)\/[^/]+/.test(pathname),
  },
  {
    domains: ["v.douyin.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname),
  },
  {
    domains: ["tiktok.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/@[^/]+\/video\/[^/]+/.test(pathname),
  },
  {
    domains: ["vm.tiktok.com", "vt.tiktok.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname),
  },
  {
    domains: ["instagram.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/(p|reel|tv)\/[^/]+/.test(pathname),
  },
  {
    domains: ["vimeo.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/\d+(?:\/|$)/.test(pathname),
  },
  {
    domains: ["player.vimeo.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/video\/\d+(?:\/|$)/.test(pathname),
  },
  {
    domains: ["x.com", "twitter.com"],
    serves: "video",
    isContentUrl: ({ pathname }) => /^\/(?:[^/]+|i)\/status\/\d+/.test(pathname),
  },
  {
    domains: ["facebook.com"],
    serves: "video",
    isContentUrl: ({ pathname, searchParams }) =>
      (pathname === "/watch" && searchParams.has("v")) ||
      /^\/(reel\/[^/]+|[^/]+\/videos\/[^/]+)/.test(pathname),
  },
  { domains: ["fb.watch"], serves: "video", isContentUrl: ({ pathname }) => /^\/[^/]+/.test(pathname) },
];

/** What a media platform address resolves to, before anything is downloaded from it. */
export function mediaPlatformContentKind(url: URL): "audio" | "video" | undefined {
  return MEDIA_PLATFORM_RULES.find(
    ({ domains, isContentUrl }) =>
      domains.some((domain) => url.hostname === domain || url.hostname.endsWith(`.${domain}`)) &&
      isContentUrl(url),
  )?.serves;
}
