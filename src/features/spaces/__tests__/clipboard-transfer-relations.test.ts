import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceClipboardPayload } from "../clipboard/clipboard-envelope";
import { readSpaceClipboardHtml, writeSpaceClipboardDirect } from "../clipboard/transfer";
import type { SpaceRelation } from "../model/relations/relation";

const clipboard = vi.hoisted(() => ({ writes: [] as string[] }));

vi.mock("tauri-plugin-clipboard-api", () => ({
  writeHtmlAndText: (html: string) => {
    clipboard.writes.push(html);
    return Promise.resolve();
  },
}));

const TRANSFORM = { x: 0, y: 0, rotation: 0, scale: 1 };
const SIZE = { width: 320, height: 180 };

function copied(): SpaceClipboardPayload {
  return {
    version: 1,
    structure: [
      { id: "s-a", parentId: null, order: 0, locked: false, kind: "content", nodeId: "a" },
      { id: "s-b", parentId: null, order: 1, locked: false, kind: "content", nodeId: "b" },
    ],
    layouts: [
      {
        structureNodeId: "s-a",
        kind: "content",
        transform: TRANSFORM,
        size: SIZE,
        sizeAuthority: "frame",
      },
      {
        structureNodeId: "s-b",
        kind: "content",
        transform: { ...TRANSFORM, x: 400 },
        size: SIZE,
        sizeAuthority: "frame",
      },
    ],
    nodes: [
      { id: "a", data: { kind: "markdown", title: "A", text: "a" } },
      { id: "b", data: { kind: "markdown", title: "B", text: "b" } },
    ],
    relations: [],
  };
}

function relation(overrides: Partial<SpaceRelation> = {}): SpaceRelation {
  return {
    id: "r-1",
    spaceId: "source-space",
    sourceId: "a",
    targetId: "b",
    kind: "supports",
    rationale: "A supports B",
    status: "active",
    statusBeforeInvalidation: null,
    actorKind: "human",
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

/** Mirrors the writer so tampered envelopes can be presented as if they came off the OS clipboard. */
function marker(value: unknown) {
  return `<!--knovum-spaces:${encodeURIComponent(JSON.stringify(value))}-->`;
}

function readWith(relations: readonly unknown[]) {
  return readSpaceClipboardHtml(marker({ ...copied(), relations }));
}

describe("structured clipboard relations", () => {
  beforeEach(() => {
    clipboard.writes.length = 0;
  });

  it("reads back the envelope the clipboard writer produced", async () => {
    const payload = { ...copied(), relations: [relation()] };

    expect(await writeSpaceClipboardDirect(payload)).toBe(true);
    expect(readSpaceClipboardHtml(clipboard.writes[0]!)).toEqual(payload);
  });

  it("keeps confidence and actor identity a copy actually carried", () => {
    expect(readWith([relation({ confidence: 0.5, actorId: "agent-1" })])?.relations).toEqual([
      relation({ confidence: 0.5, actorId: "agent-1" }),
    ]);
  });

  it("restores relation optionals the domain omitted instead of inventing nulls", () => {
    const [parsed] = readWith([relation()])?.relations ?? [];

    expect(parsed).not.toHaveProperty("confidence");
    expect(parsed).not.toHaveProperty("actorId");
    expect(parsed?.statusBeforeInvalidation).toBeNull();
  });

  it.each([
    { label: "a kind outside the relation vocabulary", tampered: { ...relation(), kind: "owns" } },
    { label: "a confidence outside the supported range", tampered: { ...relation(), confidence: 4 } },
    { label: "an actor outside the actor vocabulary", tampered: { ...relation(), actorKind: "ghost" } },
    {
      label: "an impossible lifecycle transition",
      tampered: { ...relation(), statusBeforeInvalidation: "active" },
    },
    { label: "an endpoint outside the copied content", tampered: { ...relation(), targetId: "elsewhere" } },
    { label: "a field the contract does not define", tampered: { ...relation(), injected: "x" } },
  ])("rejects a relation carrying $label", ({ tampered }) => {
    expect(readWith([tampered])).toBeUndefined();
  });

  it("rejects an envelope carrying fields the clipboard contract does not define", () => {
    expect(readSpaceClipboardHtml(marker({ ...copied(), executable: "payload" }))).toBeUndefined();
  });

  it("rejects a layout count that does not match the structure", () => {
    const payload = copied();
    payload.layouts.pop();

    expect(readSpaceClipboardHtml(marker(payload))).toBeUndefined();
  });
});
