import { beforeEach, describe, expect, it, vi } from "vitest";

import { fetchWebsiteResource } from "../website-resource";

const invoke = vi.hoisted(() => vi.fn<(command: string, args: unknown) => Promise<unknown>>());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("saved website metadata", () => {
  beforeEach(() => invoke.mockReset());

  it("keeps page identity, introduction and media from standard metadata", async () => {
    invoke.mockResolvedValue({
      resourceKind: "document",
      finalUrl: "https://example.com/articles/one",
      documentKind: "html",
      content: `<!doctype html><html><head>
        <title>Browser title</title>
        <meta property="og:title" content="Knowledge article">
        <meta property="og:site_name" content="Example Journal">
        <meta property="og:description" content="A short introduction.">
        <meta property="og:image" content="/cover.jpg">
        <link rel="icon" href="data:image/png;base64,invalid">
        <link rel="icon" href="/icon.png">
        <link rel="canonical" href="/canonical">
      </head><body><article><h1>Knowledge article</h1><p>Readable body text.</p></article></body></html>`,
    });
    const resource = await fetchWebsiteResource("https://example.com/articles/one");
    expect(resource.kind).toBe("document");
    if (resource.kind !== "document") return;
    expect(resource.metadata).toMatchObject({
      title: "Knowledge article",
      siteName: "Example Journal",
      description: "A short introduction.",
      canonicalUrl: "https://example.com/canonical",
      faviconUrl: "https://example.com/icon.png",
      previewImageUrl: "https://example.com/cover.jpg",
    });
  });
});
