import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { createVideoEditProject } from "../video-editing/project";
import { useVideoSessionStore } from "../video-editing/video-session-store";

beforeEach(() => setActivePinia(createPinia()));

it("keeps a dirty document draft on external changes or removal", () => {
  const store = useDocumentSessionStore();
  const node = createSpaceNode(createSpaceNotePayload("Original"), { x: 0, y: 0 }, "note");
  store.open({ id: "doc", kind: "document", spaceId: "space", nodeId: node.id }, node);
  const session = store.sessions.get("doc")!;
  session.state.title.value = "Draft";
  store.synchronize("doc", { ...node, data: { ...node.data, title: "External" } });
  expect(session.state.title.value).toBe("Draft");
  expect(session.node.value.data.title).toBe("Original");
  expect(session.error.value).toBe("conflict");
  store.synchronize("doc", undefined);
  expect(session.error.value).toBe("missing");
  expect(session.state.title.value).toBe("Draft");
});

it("reloads clean document content and title without marking them dirty", () => {
  const store = useDocumentSessionStore();
  const node = createSpaceNode(createSpaceNotePayload("Original"), { x: 0, y: 0 }, "note");
  store.open({ id: "doc", kind: "document", spaceId: "space", nodeId: node.id }, node);
  const session = store.sessions.get("doc")!;
  store.synchronize("doc", {
    ...node,
    data: {
      ...node.data,
      title: "Renamed",
      plainText: "Changed",
      tiptapContent: {
        type: "doc",
        content: [{ type: "paragraph", content: [{ type: "text", text: "Changed" }] }],
      },
    },
  });
  expect(session.state.title.value).toBe("Renamed");
  expect(session.contentVersion.value).toBe(1);
  expect(session.dirty.value).toBe(false);
});

it("retains independent video drafts, histories and positions", () => {
  const store = useVideoSessionStore();
  const saved = createVideoEditProject({
    sourceRevision: "r1",
    durationMs: 1000,
    sourceWidth: 640,
    sourceHeight: 480,
  });
  const node = createSpaceNode(
    {
      kind: "video",
      title: "Video",
      fileName: "v.mp4",
      assetPath: "/v.mp4",
      sourceRevision: "r1",
      videoEditProject: saved,
    },
    { x: 0, y: 0 },
    "v",
  );
  store.open({ id: "a", kind: "video", spaceId: "space", nodeId: "v" }, node);
  const session = store.sessions.get("a")!;
  session.history.commit({
    ...session.project.value!,
    textTrack: { items: [] },
    videoTrack: { clips: [{ ...saved.videoTrack.clips[0]!, volume: 0 }] },
  });
  session.timelineMs.value = 420;
  session.timelineZoom.value = 2;
  store.synchronize("a", { ...node, data: { ...node.data, sourceRevision: "r2" } });
  expect(session.error.value).toBe("conflict");
  expect(session.history.canUndo.value).toBe(true);
  expect(session.timelineMs.value).toBe(420);
  expect(session.timelineZoom.value).toBe(2);
  expect(saved.videoTrack.clips[0]!.volume).toBe(1);
  session.history.undo();
  expect(session.project.value).toEqual(saved);
  store.release("a");
  expect(store.sessions.size).toBe(0);
});
