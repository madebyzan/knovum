import { describe, expect, it } from "vitest";

import { normalizeSpaceUrl, websiteDefaultTitle } from "../website/url";

describe("normalizeSpaceUrl", () => {
  it("keeps http and https URLs", () => {
    expect(normalizeSpaceUrl("https://example.com/path")).toBe("https://example.com/path");
    expect(normalizeSpaceUrl("http://example.com")).toBe("http://example.com/");
  });

  it("turns a domain-like address into https", () => {
    expect(normalizeSpaceUrl("github.com/openai")).toBe("https://github.com/openai");
  });

  it("does not turn ordinary text or unsafe schemes into websites", () => {
    expect(normalizeSpaceUrl("project notes")).toBeUndefined();
    expect(normalizeSpaceUrl("javascript:alert(1)")).toBeUndefined();
    expect(normalizeSpaceUrl("file:///tmp/test")).toBeUndefined();
  });
});

describe("websiteDefaultTitle", () => {
  it("uses the readable hostname", () => {
    expect(websiteDefaultTitle("https://www.example.com/path")).toBe("example.com");
  });
});
