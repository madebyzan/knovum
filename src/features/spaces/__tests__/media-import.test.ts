import { describe, expect, it } from "vitest";

import { knowledgeRevision } from "../knowledge/model/revision";
import { importedMediaSchema, mediaImportProgressSchema } from "../media/import/contract";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { classifySpaceUrl } from "../sources/classify-url";

describe("Space media import contracts", () => {
  it.each([
    "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    "https://www.bilibili.com/video/BV1xx411c7mD",
    "https://vimeo.com/76979871",
    "https://example.com/article",
  ])("normalizes HTTP sources without duplicating yt-dlp extractors", (value) => {
    expect(classifySpaceUrl(value)).toEqual({ kind: "url", url: value });
  });

  it("leaves non-URL text to note capture", () => {
    expect(classifySpaceUrl("not a URL")).toBeUndefined();
  });

  it("accepts web origin metadata on standard media nodes", () => {
    expect(
      validateSpaceNodePayload({
        kind: "video",
        title: "The New Vimeo Player",
        origin: {
          kind: "web",
          url: "https://vimeo.com/76979871",
          provider: "Vimeo",
          externalId: "76979871",
        },
        assetPath: "/spaces/assets/video.mp4",
        fileName: "video.mp4",
        size: 1024,
        modifiedAt: 1,
        sourceRevision: "asset1",
      }),
    ).toMatchObject({ kind: "video", origin: { provider: "Vimeo" } });
  });

  it("matches the strict local import returned by Rust", () => {
    const imported = {
      kind: "video",
      title: "Video",
      creator: null,
      durationSeconds: null,
      intrinsicWidth: 1920,
      intrinsicHeight: 1080,
      assetPath: "/spaces/assets/video.mp4",
      fileName: "video.mp4",
      size: 1024,
      modifiedAt: 1,
      sourceRevision: "asset1",
      previewPath: null,
      origin: {
        kind: "web",
        url: "https://example.com/watch/42",
        provider: "Generic",
        externalId: "42",
      },
    };
    expect(importedMediaSchema.parse(imported)).toEqual(imported);
    expect(() => importedMediaSchema.parse({ ...imported, creator: undefined })).toThrow("Invalid input");
  });

  it("accepts only the strict local import progress contract", () => {
    const progress = {
      stage: "downloading",
      downloadedBytes: 512,
      totalBytes: 1024,
      processedMs: null,
      durationMs: null,
    } as const;
    expect(mediaImportProgressSchema.parse(progress)).toEqual(progress);
    expect(() => mediaImportProgressSchema.parse({ ...progress, percent: 50 })).toThrow("Unrecognized key");
  });

  it("includes media origin and source content in the Knowledge revision", () => {
    const data = {
      kind: "audio",
      title: "Track",
      creator: "Artist",
      durationSeconds: 90,
      assetPath: "/spaces/assets/track.m4a",
      fileName: "track.m4a",
      size: 1024,
      modifiedAt: 1,
      sourceRevision: "asset1",
      origin: {
        kind: "web",
        url: "https://soundcloud.com/a/b",
        provider: "Soundcloud",
        externalId: "42",
      },
    } as const satisfies SpaceNodePayload;
    expect(knowledgeRevision({ ...data, sourceRevision: "asset2" })).not.toBe(knowledgeRevision(data));
    expect(knowledgeRevision({ ...data, origin: { ...data.origin, externalId: "43" } })).not.toBe(
      knowledgeRevision(data),
    );
  });
});
