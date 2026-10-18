import { describe, expect, it } from "vitest";

import { classifyUrlSource } from "../url-source-kind";

describe("URL source classification", () => {
  it.each([
    ["https://example.com/video.mp4?token=42", "video"],
    ["https://example.com/talk.mp3", "audio"],
    ["https://www.youtube.com/watch?v=abc123", "video"],
    ["https://www.bilibili.com/audio/au123456", "audio"],
    ["https://www.youtube.com/@creator", "website"],
    ["https://www.chinagoods.com/", "website"],
  ] as const)("classifies %s as %s", (input, kind) => {
    expect(classifyUrlSource(new URL(input))).toBe(kind);
  });
});
