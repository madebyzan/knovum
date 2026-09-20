import { describe, expect, it } from "vitest";

import { isDirectMediaResource } from "../direct-media-resource";

describe("direct media resource URL", () => {
  it("recognizes direct file paths without interpreting query strings as extensions", () => {
    expect(isDirectMediaResource(new URL("https://example.com/video.mp4?token=42"))).toBe(true);
    expect(isDirectMediaResource(new URL("https://example.com/article?download=video.mp4"))).toBe(false);
  });
});
