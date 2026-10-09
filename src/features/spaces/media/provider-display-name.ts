const DISPLAY_NAME_BY_PROVIDER = new Map([
  ["Youtube", "YouTube"],
  ["BiliBili", "Bilibili"],
  ["Douyin", "抖音"],
  ["TikTok", "TikTok"],
  ["Instagram", "Instagram"],
  ["Vimeo", "Vimeo"],
  ["Twitter", "X"],
  ["Facebook", "Facebook"],
]);

export function mediaProviderDisplayName(provider: string): string {
  return DISPLAY_NAME_BY_PROVIDER.get(provider) ?? provider;
}
