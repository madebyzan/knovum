import { Markdown } from "@tiptap/markdown";
import { StarterKit } from "@tiptap/starter-kit";
import { Editor } from "@tiptap/vue-3";
import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import SpaceTiptapWritingMenu from "../presentation/SpaceTiptapWritingMenu.vue";
import { ReasoningTrace } from "../tiptap-reasoning-trace";
import { useSpaceTiptapWritingTools } from "../use-space-tiptap-writing-tools";

const ai = vi.hoisted(() => ({
  transform: vi.fn<(...args: unknown[]) => Promise<string>>(),
  cancel: vi.fn<(jobId: string) => Promise<void>>(),
  error: vi.fn<(...args: unknown[]) => void>(),
}));
vi.mock("../../../../foundation/ai/editor-transform", () => ({ transformEditorTextStream: ai.transform }));
vi.mock("../../../../foundation/ai/ai-runtime", () => ({ cancelAiJob: ai.cancel }));
vi.mock("vue-sonner", () => ({ toast: { error: ai.error, info: vi.fn<() => void>() } }));
vi.mock("vue-i18n", async (original) => ({
  ...(await original<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

const cleanups: Array<() => void> = [];
function deferred<T>() {
  let accept!: (value: T) => void;
  let fail!: (cause: unknown) => void;
  const promise = new Promise<T>((resolve, reject) => {
    accept = resolve;
    fail = reject;
  });
  return { promise, resolve: accept, reject: fail };
}
function session() {
  const editor = new Editor({
    extensions: [StarterKit, Markdown, ReasoningTrace],
    content: "<p>hello world</p>",
  });
  editor.commands.setTextSelection({ from: 1, to: 6 });
  const scope = effectScope();
  const writing = scope.run(() => useSpaceTiptapWritingTools(shallowRef(editor)))!;
  cleanups.push(() => {
    scope.stop();
    editor.destroy();
  });
  return { editor, scope, writing };
}

beforeEach(() => {
  ai.transform.mockReset();
  ai.cancel.mockReset().mockResolvedValue(undefined);
  ai.error.mockClear();
});
afterEach(() => {
  for (const cleanup of cleanups.splice(0)) cleanup();
});

describe("editor writing task ownership", () => {
  it("applies a completed task and releases the busy state", async () => {
    ai.transform.mockResolvedValue("improved");
    const { editor, writing } = session();
    await writing.run("improve");
    expect(editor.getText()).toBe("improved\n\n world");
    expect(writing.runningAction.value).toBeUndefined();
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

  it("does not report a cancelled request rejection as a writing failure", async () => {
    const completion = deferred<string>();
    ai.transform.mockReturnValue(completion.promise);
    const { writing } = session();
    const running = writing.run("improve");
    await writing.cancel();
    completion.reject(new Error("cancelled"));
    await running;
    expect(ai.error).not.toHaveBeenCalled();
    expect(writing.runningAction.value).toBeUndefined();
  });

  it("does not let a cancelled task write or clear a newer task", async () => {
    const first = deferred<string>();
    const second = deferred<string>();
    ai.transform.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const { editor, writing } = session();
    const previous = writing.run("improve");
    await writing.cancel();
    const current = writing.run("extend");
    first.resolve("obsolete");
    await previous;
    expect(editor.getText()).toBe("hello world");
    expect(writing.runningAction.value).toBe("extend");
    second.resolve("current");
    await current;
    expect(editor.getText()).toBe("current\n\n world");
    expect(writing.runningAction.value).toBeUndefined();
  });

  it("disables selection writing while the editor task runs", async () => {
    const completion = deferred<string>();
    ai.transform.mockReturnValue(completion.promise);
    const { writing } = session();
    const bubble = mount(SpaceTiptapWritingMenu, { props: { writing, compact: true } });
    cleanups.push(() => {
      bubble.unmount();
    });
    const running = writing.run("improve");
    await nextTick();
    expect(bubble.get("button").attributes("disabled")).toBeDefined();
    completion.resolve("improved");
    await running;
    await nextTick();
    expect(bubble.get("button").attributes("disabled")).toBeUndefined();
  });
});
