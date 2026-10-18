import { describe, expect, it } from "vitest";

import { directMediaResourceKind } from "../direct-media-resource";

describe("direct media resource URL", () => {
  it("recognizes direct file paths without interpreting query strings as extensions", () => {
    expect(directMediaResourceKind(new URL("https://example.com/video.mp4?token=42"))).toBe("video");
    expect(directMediaResourceKind(new URL("https://example.com/talk.mp3"))).toBe("audio");
    expect(directMediaResourceKind(new URL("https://example.com/article?download=video.mp4"))).toBe(
      undefined,
    );
  });
});
