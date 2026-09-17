const PROVIDER_NAMES: Readonly<Record<string, string>> = {
  Youtube: "YouTube",
  BiliBili: "Bilibili",
  Douyin: "抖音",
  TikTok: "TikTok",
  Instagram: "Instagram",
  Vimeo: "Vimeo",
  Twitter: "X",
  Facebook: "Facebook",
};

export function mediaProviderDisplayName(provider: string): string {
  return PROVIDER_NAMES[provider] ?? provider;
}
