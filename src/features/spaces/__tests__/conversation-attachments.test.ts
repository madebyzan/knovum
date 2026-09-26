import { beforeEach, expect, it, vi } from "vitest";

const uploads = vi.hoisted(() => vi.fn<() => Promise<{ path: string; sourceRevision: string }[]>>());
vi.mock("../conversation/conversation-uploads", () => ({ chooseAssistantUploads: uploads }));
vi.mock("../canvas/editor-session", () => ({ useCanvasEditorSession: () => ({}) }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => ({}) }));
import { normalizeCommandError } from "../../../foundation/ipc/command-error";
import { useConversationAttachments } from "../conversation/use-conversation-attachments";

beforeEach(() => {
  uploads.mockReset();
});

it("rejects more than 32 attachments visibly and preserves the previous selection", async () => {
  const session = useConversationAttachments();
  uploads.mockResolvedValue([{ path: "/existing.txt", sourceRevision: "revision" }]);
  await session.chooseFiles(() => true);
  const existing = session.attachments.value;
  uploads.mockResolvedValue(
    Array.from({ length: 33 }, (_, index) => ({
      path: `/file-${index}.txt`,
      sourceRevision: "revision",
    })),
  );
  await session.chooseFiles(() => true);
  expect(session.attachmentError.value).toBe("spaces.assistant.attachmentLimit");
  expect(session.attachments.value).toBe(existing);
  expect(session.pickingFiles.value).toBe(false);
});

it("selects the localized outcome by wire code even when the backend message changes", async () => {
  const session = useConversationAttachments();
  uploads.mockRejectedValue(
    normalizeCommandError({
      code: "assistant_upload_unauthorized",
      message: "外部文件访问已拒绝",
    }),
  );
  await session.chooseFiles(() => true);
  expect(session.attachmentError.value).toBe("spaces.assistant.uploadPermissionError");
  expect(session.attachments.value).toEqual([]);
});

it("does not publish a late selection into a different conversation", async () => {
  const session = useConversationAttachments();
  uploads.mockResolvedValue([{ path: "/late.txt", sourceRevision: "revision" }]);
  await session.chooseFiles(() => false);
  expect(session.attachments.value).toEqual([]);
  expect(session.attachmentError.value).toBeUndefined();
  expect(session.pickingFiles.value).toBe(false);
});
