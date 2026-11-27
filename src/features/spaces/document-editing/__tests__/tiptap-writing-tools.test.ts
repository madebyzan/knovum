import { Markdown } from "@tiptap/markdown";
import { StarterKit } from "@tiptap/starter-kit";
import { Editor } from "@tiptap/vue-3";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import type { EditorTextTransformInput } from "../../../../foundation/ai/editor-transform";
import { deferred } from "../../../../shared/__tests__/support/deferred";
import { ReasoningTrace } from "../tiptap-reasoning-trace";
import { useSpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";

const ai = vi.hoisted(() => ({
  transform: vi.fn<(input: EditorTextTransformInput) => Promise<string>>(),
  cancel: vi.fn<(jobId: string) => Promise<void>>(),
  error: vi.fn<(...args: unknown[]) => void>(),
  admit: vi.fn<(...args: unknown[]) => Promise<boolean>>(),
}));
vi.mock("../../../../foundation/ai/ai-readiness-admission", () => ({ admitAiRequirements: ai.admit }));
vi.mock("../../../../foundation/ai/editor-transform", () => ({ transformEditorTextStream: ai.transform }));
vi.mock("../../../../foundation/ai/ai-runtime", () => ({ cancelAiJob: ai.cancel }));
vi.mock("vue-sonner", () => ({ toast: { error: ai.error, info: vi.fn<() => void>() } }));
vi.mock("vue-i18n", async (original) => ({
  ...(await original<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

const cleanups: Array<() => void> = [];
function session() {
  const editor = new Editor({
    extensions: [StarterKit, Markdown, ReasoningTrace],
    content: "<p>hello world</p>",
  });
  editor.commands.setTextSelection({ from: 1, to: 6 });
  const scope = effectScope();
  const writing = scope.run(() =>
    useSpaceTiptapWritingTools(shallowRef(editor), { documentTitle: shallowRef("Spec") }),
  )!;
  cleanups.push(() => {
    scope.stop();
    editor.destroy();
  });
  return { editor, scope, writing };
}

function emptySession(initialTitle: string, promoteTitle?: (heading: string) => boolean) {
  const editor = new Editor({ extensions: [StarterKit, Markdown, ReasoningTrace], content: "<p></p>" });
  editor.commands.setTextSelection(1);
  const title = shallowRef(initialTitle);
  const scope = effectScope();
  const writing = scope.run(() =>
    useSpaceTiptapWritingTools(shallowRef(editor), { documentTitle: title, promoteTitle }),
  )!;
  cleanups.push(() => {
    scope.stop();
    editor.destroy();
  });
  return { editor, title, writing };
}

beforeEach(() => {
  ai.transform.mockReset();
  ai.cancel.mockReset().mockResolvedValue(undefined);
  ai.error.mockClear();
  ai.admit.mockReset().mockResolvedValue(true);
});
afterEach(() => {
  for (const cleanup of cleanups.splice(0)) cleanup();
});

describe("editor streaming AI co-writing session", () => {
  it("streams live deltas and replaces inline selection without splitting the paragraph", async () => {
    ai.transform.mockImplementation(async (input) => {
      input.onEvent({ event: "thinking_delta", text: "planning" });
      input.onEvent({ event: "delta", text: "im" });
      input.onEvent({ event: "delta", text: "proved" });
      return "improved";
    });
    const { editor, writing } = session();
    await writing.run("improve");

    expect(writing.phase.value).toBe("review");
    expect(writing.draftText.value).toBe("improved");
    expect(editor.getText()).toBe("hello world");
    expect(ai.admit).toHaveBeenCalledWith([{ kind: "routed", purpose: "writing" }]);

    writing.acceptReplace();
    expect(editor.getText()).toBe("improved world");
    expect(writing.phase.value).toBe("closed");
  });

  it("inserts generated draft below the block when accepting non-destructively", async () => {
    ai.transform.mockResolvedValue("Extra paragraph");
    const { editor, writing } = session();
    await writing.run("improve");
    writing.acceptInsertBelow();
    expect(editor.getText()).toBe("hello world\n\nExtra paragraph");
  });

  it("supports custom prompt and follow-up refinement with previousDraft", async () => {
    ai.transform.mockResolvedValueOnce("First draft").mockResolvedValueOnce("Shorter draft");
    const { writing } = session();
    writing.openPrompt();
    writing.promptText.value = "Draft a summary";
    await writing.submitPrompt();
    expect(writing.draftText.value).toBe("First draft");

    writing.promptText.value = "Make it shorter";
    await writing.submitPrompt();
    expect(ai.transform).toHaveBeenLastCalledWith(
      expect.objectContaining({
        action: "custom",
        instruction: "Make it shorter",
        previousDraft: "First draft",
        context: expect.objectContaining({ documentTitle: "Spec" }),
      }),
    );
    expect(writing.draftText.value).toBe("Shorter draft");
  });

  it("does not run when AI admission fails and closes cleanly", async () => {
    ai.admit.mockResolvedValue(false);
    const { editor, writing } = session();
    await writing.run("improve");
    expect(ai.transform).not.toHaveBeenCalled();
    expect(editor.getText()).toBe("hello world");
    expect(writing.phase.value).toBe("closed");
  });

  it("cancels on scope disposal and rejects late completion", async () => {
    const completion = deferred<string>();
    ai.transform.mockReturnValue(completion.promise);
    const { editor, scope, writing } = session();
    const running = writing.run("improve");
    scope.stop();
    completion.resolve("obsolete");
    await running;
    expect(editor.getText()).toBe("hello world");
    expect(ai.cancel).toHaveBeenCalledOnce();
    expect(ai.error).not.toHaveBeenCalled();
  });

  it("tracks runningAction while streaming and clears when finished", async () => {
    const completion = deferred<string>();
    ai.transform.mockReturnValue(completion.promise);
    const { writing } = session();
    const running = writing.run("improve");
    expect(writing.runningAction.value).toBe("improve");
    expect(writing.phase.value).toBe("streaming");
    completion.resolve("improved");
    await running;
    expect(writing.runningAction.value).toBeUndefined();
    expect(writing.phase.value).toBe("review");
  });

  it("parses structured Markdown (headings, lists, mermaid fence) and unwraps outer markdown fences", async () => {
    const { editor, writing } = emptySession("计划");
    ai.transform.mockResolvedValue(
      "```markdown\n## 12月份计划\n\n1. 核心重构\n2. 灰度上线\n\n```mermaid\nflowchart LR\n  A[需求] --> B[开发]\n```\n```",
    );
    writing.openPrompt();
    writing.promptText.value = "生成12月份计划并附流程图";
    await writing.submitPrompt();
    expect(writing.draftText.value).not.toMatch(/^```markdown/u);
    expect(writing.draftText.value).toContain("```mermaid");

    writing.acceptReplace();
    const blocks = editor.getJSON().content ?? [];
    expect(blocks[0]?.type).toBe("heading");
    expect(blocks[1]?.type).toBe("orderedList");
    expect(blocks[2]?.type).toBe("codeBlock");
    expect(blocks[2]?.attrs?.language).toBe("mermaid");
  });

  it("promotes a leading H1 into the document title when accepting into an empty untitled document", async () => {
    const promoted = shallowRef("未命名 Markdown");
    const { editor, writing } = emptySession("未命名 Markdown", (heading) => {
      if (promoted.value !== "未命名 Markdown") return false;
      promoted.value = heading;
      return true;
    });
    ai.transform.mockResolvedValue("# 月亮的小船\n\n夜风吹过湖面，一只小船亮起微光。");
    writing.openPrompt();
    writing.promptText.value = "写一个睡前故事";
    await writing.submitPrompt();
    writing.acceptReplace();

    expect(promoted.value).toBe("月亮的小船");
    expect(editor.getText()).toBe("夜风吹过湖面，一只小船亮起微光。");
    expect(editor.getJSON().content?.[0]?.type).toBe("paragraph");
  });
});
