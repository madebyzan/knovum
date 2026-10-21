import type { error as logError } from "@tauri-apps/plugin-log";
import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, effectScope, shallowRef } from "vue";

import { i18n } from "../../../../foundation/i18n/instance";
import type { SpeechSignalReport } from "../../contracts/generated/SpeechSignalReport";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { ImportedAudiobookScript } from "../audiobook-script";

const mocks = vi.hoisted(() => ({
  cancel: vi.fn<(jobId: string) => Promise<unknown>>(),
  inspect: vi.fn<() => Promise<SpeechSignalReport>>(),
  logError: vi.fn<typeof logError>(),
  readScript: vi.fn<() => Promise<ImportedAudiobookScript>>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("../ipc", () => ({ cancelSpeechGeneration: mocks.cancel }));
vi.mock("../signal-inspection-ipc", () => ({ inspectSpeechSignal: mocks.inspect }));
vi.mock("../audiobook-script-import", () => ({ readNodeAudiobookScript: mocks.readScript }));

import { deferred } from "../../../../shared/__tests__/support/deferred";
import { useAudiobookScriptImport } from "../use-audiobook-script-import";
import { useSpeechSignalInspection } from "../use-speech-signal-inspection";

const scopes: ReturnType<typeof effectScope>[] = [];

beforeEach(() => {
  mocks.cancel.mockReset();
  mocks.cancel.mockResolvedValue(undefined);
  mocks.inspect.mockReset();
  mocks.logError.mockReset();
  mocks.logError.mockResolvedValue();
  mocks.readScript.mockReset();
});

afterEach(() => {
  scopes.splice(0).forEach((scope) => scope.stop());
});

describe("speech job lifecycle", () => {
  it("starts a new inspection immediately after reset and logs the stale cancellation failure", async () => {
    const oldInspection = deferred<SpeechSignalReport>();
    const oldCancellation = deferred<never>();
    mocks.inspect
      .mockReturnValueOnce(oldInspection.promise)
      .mockRejectedValueOnce(new Error("current inspection failed"));
    mocks.cancel.mockReturnValueOnce(oldCancellation.promise);
    const projectId = shallowRef("project");
    const revisionId = shallowRef("revision-1");
    const inspection = setup(() => useSpeechSignalInspection({ projectId, revisionId }));

    const oldTask = inspection.inspect();
    await vi.waitFor(() => expect(mocks.inspect).toHaveBeenCalledOnce());
    revisionId.value = "revision-2";
    expect(inspection.jobId.value).toBeUndefined();
    await inspection.inspect();
    expect(mocks.inspect).toHaveBeenCalledTimes(2);
    expect(inspection.error.value).toBe("current inspection failed");

    oldCancellation.reject(new Error("stale cancellation failed"));
    await vi.waitFor(() => expect(mocks.logError).toHaveBeenCalledOnce());
    expect(inspection.error.value).toBe("current inspection failed");
    oldInspection.resolve(report(1));
    expect(await oldTask).toBeUndefined();
  });

  it("logs a cancellation failure after its inspection scope is disposed", async () => {
    const pendingInspection = deferred<SpeechSignalReport>();
    const pendingCancellation = deferred<never>();
    mocks.inspect.mockReturnValueOnce(pendingInspection.promise);
    mocks.cancel.mockReturnValueOnce(pendingCancellation.promise);
    const scope = effectScope();
    scopes.push(scope);
    const inspection = scope.run(() =>
      useSpeechSignalInspection({
        projectId: shallowRef("project"),
        revisionId: shallowRef("revision"),
      }),
    )!;
    const task = inspection.inspect();
    await vi.waitFor(() => expect(mocks.inspect).toHaveBeenCalledOnce());

    scope.stop();
    pendingCancellation.reject(new Error("dispose cancellation failed"));
    await vi.waitFor(() => expect(mocks.logError).toHaveBeenCalledOnce());
    expect(inspection.error.value).toBeUndefined();
    pendingInspection.resolve(report(2));
    expect(await task).toBeUndefined();
  });

  it("invalidates an audiobook import when the source revision changes", async () => {
    const oldImport = deferred<ImportedAudiobookScript>();
    const oldCancellation = deferred<never>();
    const currentScript = script("current");
    mocks.readScript.mockReturnValueOnce(oldImport.promise).mockResolvedValueOnce(currentScript);
    mocks.cancel.mockReturnValueOnce(oldCancellation.promise);
    const node = shallowRef(documentNode("revision-1"));
    let importer!: ReturnType<typeof useAudiobookScriptImport>;
    const wrapper = mount(
      defineComponent({
        setup() {
          importer = useAudiobookScriptImport(node);
          return () => null;
        },
      }),
      { global: { plugins: [i18n] } },
    );
    onTestFinished(() => wrapper.unmount());

    const oldTask = importer.importScript();
    await vi.waitFor(() => expect(mocks.readScript).toHaveBeenCalledOnce());
    node.value = documentNode("revision-2");
    expect(importer.importing.value).toBe(false);
    expect(await importer.importScript()).toEqual(currentScript);

    oldCancellation.reject(new Error("old import cancellation failed"));
    await vi.waitFor(() => expect(mocks.logError).toHaveBeenCalledOnce());
    oldImport.resolve(script("old"));
    expect(await oldTask).toBeUndefined();
    expect(importer.script.value).toEqual(currentScript);
    expect(importer.error.value).toBeUndefined();
  });
});

function setup<T>(factory: () => T) {
  const scope = effectScope();
  scopes.push(scope);
  return scope.run(factory)!;
}

function report(scannedDurationMs: number): SpeechSignalReport {
  return { scannedDurationMs, truncated: false, warnings: [] };
}

function script(text: string): ImportedAudiobookScript {
  return {
    schemaVersion: 1,
    chapters: [{ title: "Chapter", paragraphs: [{ role: "Narrator", text }] }],
  };
}

function documentNode(sourceRevision: string): SpaceCanvasNode {
  return {
    id: "document",
    type: "file",
    initialPosition: { x: 0, y: 0 },
    data: {
      kind: "file",
      title: "Manuscript",
      assetPath: "/managed/book.pdf",
      fileName: "book.pdf",
      sourceRevision,
    },
  };
}
