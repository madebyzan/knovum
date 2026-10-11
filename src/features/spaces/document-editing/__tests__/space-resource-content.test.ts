import { describe, expect, it } from "vitest";

import { validateSpaceNodePayload } from "../../model/nodes/node-validation";
import { createSpaceNotePayload } from "../tiptap-content";

describe("spaces resource content validation", () => {
  it("validates audio as a private opaque asset", () => {
    expect(
      validateSpaceNodePayload({
        kind: "audio",
        title: "Interview.m4a",
        fileName: "Interview.m4a",
        assetPath: "/private/spaces/assets/interview.m4a",
      }),
    ).toMatchObject({ kind: "audio" });
  });

  it("validates source code, but rejects directory and archive nodes", () => {
    expect(
      validateSpaceNodePayload({
        kind: "code",
        title: "index.ts",
        fileName: "index.ts",
        assetPath: "/private/spaces/assets/index.ts",
        previewText: "export const answer = 42;",
        previewLanguage: "typescript",
        size: 25,
        modifiedAt: 1,
      }),
    ).toMatchObject({ kind: "code", previewLanguage: "typescript" });
    expect(() =>
      validateSpaceNodePayload({
        kind: "folder",
        title: "src",
        fileName: "src",
        assetPath: "/private/spaces/assets/src",
        previewText: "index.ts",
        entries: [{ path: "index.ts", kind: "file", size: 25 }],
        size: 25,
        modifiedAt: 1,
      }),
    ).toThrow("Invalid discriminator value");
    expect(() =>
      validateSpaceNodePayload({
        kind: "archive",
        title: "bundle.zip",
        fileName: "bundle.zip",
        assetPath: "/private/spaces/assets/bundle.zip",
        previewText: "src/index.ts",
        entries: [{ path: "src/index.ts", kind: "file", size: 25 }],
        size: 100,
        modifiedAt: 1,
      }),
    ).toThrow("Invalid discriminator value");
  });

  it("validates file with archive directory entries", () => {
    expect(
      validateSpaceNodePayload({
        kind: "file",
        title: "bundle.zip",
        fileName: "bundle.zip",
        assetPath: "/private/spaces/assets/bundle.zip",
        previewText: "src/index.ts",
        entries: [{ path: "src/index.ts", kind: "file", size: 25 }],
        size: 100,
        modifiedAt: 1,
      }),
    ).toMatchObject({
      kind: "file",
      entries: [{ path: "src/index.ts", kind: "file", size: 25 }],
    });
  });

  it("rejects resource metadata on unrelated node kinds", () => {
    expect(() =>
      validateSpaceNodePayload({
        kind: "website",
        title: "Example",
        url: "https://example.com",
        previewText: "not website state",
      }),
    ).toThrow("Unrecognized key");
    expect(() =>
      validateSpaceNodePayload({
        ...createSpaceNotePayload("Note"),
        assetPath: "/private/spaces/assets/file",
      }),
    ).toThrow("Unrecognized key");
  });
});
