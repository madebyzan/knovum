import { describe, expect, it } from "vitest";

import { isMediaPlatformContentUrl } from "../media-platform-url";

describe("automatic media platform routing", () => {
  it.each([
    "https://www.youtube.com/watch?v=abc123",
    "https://www.youtube.com/watch?v=CmjNjWeRE0Y",
    "https://youtu.be/abc123",
    "https://www.bilibili.com/video/BV1xx411c7mu",
    "https://b23.tv/abc123",
    "https://www.douyin.com/video/123456",
    "https://v.douyin.com/abc123",
    "https://www.tiktok.com/@creator/video/123456",
    "https://www.instagram.com/reel/abc123/",
    "https://vimeo.com/123456789",
    "https://x.com/creator/status/123456789",
    "https://www.facebook.com/watch?v=123456789",
  ])("routes a recognized content URL to media acquisition: %s", (url) => {
    expect(isMediaPlatformContentUrl(new URL(url))).toBe(true);
  });

  it.each([
    "https://www.chinagoods.com/",
    "https://www.youtube.com/",
    "https://www.youtube.com/@creator",
    "https://www.bilibili.com/",
    "https://www.douyin.com/user/creator",
    "https://www.instagram.com/creator/",
    "https://x.com/creator",
    "https://youtube.com.evil.example/watch?v=abc123",
    "https://notbilibili.com/video/BV123",
  ])("keeps ordinary websites and platform profiles as websites: %s", (url) => {
    expect(isMediaPlatformContentUrl(new URL(url))).toBe(false);
  });
});
