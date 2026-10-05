import { beforeEach, describe, expect, it, vi } from "vitest";

import { chooseAudiobookScript } from "../audiobook-script-import";

const mocks = vi.hoisted(() => ({
  open: vi.fn<(options: unknown) => Promise<string | null>>(),
  invoke: vi.fn<(command: string, arguments_?: unknown) => Promise<unknown>>(),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.open }));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));

beforeEach(() => {
  mocks.open.mockReset();
  mocks.invoke.mockReset();
});
describe("audiobook document import ownership", () => {
  it("does not start native extraction after the owning editor has closed its file dialog", async () => {
    let completeDialog!: (value: string) => void;
    const promise = new Promise<string>((resolve) => {
      completeDialog = resolve;
    });
    let current = true;
    mocks.open.mockReturnValue(promise);
    const pending = chooseAudiobookScript("Narrator", "job", () => current);
    current = false;
    completeDialog("/book.docx");
    expect(await pending).toBeUndefined();
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
  it("turns native document text into owned chapter and paragraph contracts", async () => {
    mocks.open.mockResolvedValue("/book.pdf");
    mocks.invoke.mockResolvedValue({
      kind: "text",
      text: "# Chapter one\n\nFirst paragraph.\n\nSecond paragraph.",
    });
    const script = await chooseAudiobookScript("Narrator", "job", () => true);
    expect(script?.chapters).toEqual([
      {
        title: "Chapter one",
        paragraphs: [
          { role: "Narrator", text: "First paragraph." },
          { role: "Narrator", text: "Second paragraph." },
        ],
      },
    ]);
  });
  it("rejects structured scripts with unknown contract fields", async () => {
    mocks.open.mockResolvedValue("/book.json");
    mocks.invoke.mockResolvedValue({
      kind: "text",
      text: JSON.stringify({
        schemaVersion: 1,
        chapters: [{ title: "Chapter", paragraphs: [{ role: "Narrator", text: "Hello" }] }],
        legacy: true,
      }),
    });
    await expect(chooseAudiobookScript("Narrator", "job", () => true)).rejects.toThrow(/Unrecognized key/u);
  });
});
