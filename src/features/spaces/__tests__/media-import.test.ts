import { describe, expect, it } from "vitest";

import {
  importedMediaSchema,
  mediaAcquisitionEventSchema,
  mediaImportProgressSchema,
} from "../media/import/schema/import";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
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

  /** The canvas reserves its node from the resolved event, so both events are one contract. */
  it("reads what the acquisition reports: the identified source, then its progress", () => {
    const resolved = {
      event: "resolved",
      kind: "audio",
      title: "Interview",
      durationSeconds: 1800,
      assetPath: "/spaces/assets/import.m4a",
    } as const;
    expect(mediaAcquisitionEventSchema.parse(resolved)).toEqual(resolved);
    const progress = {
      event: "progress",
      stage: "finalizing",
      downloadedBytes: 0,
      totalBytes: null,
      processedMs: 1200,
      durationMs: 1800,
    } as const;
    expect(mediaAcquisitionEventSchema.parse(progress)).toEqual(progress);
    expect(() => mediaAcquisitionEventSchema.parse({ ...resolved, assetPath: "" })).toThrow("Too small");
    expect(() => mediaAcquisitionEventSchema.parse({ ...resolved, event: "named" })).toThrow(
      "No matching discriminator",
    );
  });
});
