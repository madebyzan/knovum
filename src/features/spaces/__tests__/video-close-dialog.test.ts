import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";

import { bindCommandShortcuts } from "../../../foundation/commands/command-shortcuts";
import { runAppCommand } from "../../../foundation/commands/run-app-command";
import { i18n } from "../../../foundation/i18n/instance";
import SpaceVideoUnsavedDialog from "../presentation/video-editor/SpaceVideoUnsavedDialog.vue";

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "windows" }));

it("cancels its own decision with Escape without closing the video behind the modal", async () => {
  const closeOtherVideo = vi.fn<() => void>();
  const command = {
    id: "video-edit.close",
    title: "Close",
    shortcut: "Escape",
    category: "editing",
    run: closeOtherVideo,
  } as const;
  const wrapper = mount(SpaceVideoUnsavedDialog, {
    props: { open: true, saving: false, canSave: true },
    attachTo: document.body,
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  onTestFinished(bindCommandShortcuts([command]));
  await flushPromises();
  runAppCommand(command);
  await flushPromises();
  expect(closeOtherVideo).not.toHaveBeenCalled();
  const dialog = document.querySelector<HTMLElement>('[role="dialog"][data-state="open"]')!;
  const button = dialog.querySelector<HTMLButtonElement>("button")!;
  button.focus();
  button.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true, cancelable: true }),
  );
  await flushPromises();
  expect(wrapper.emitted("cancel")).toHaveLength(1);
  expect(closeOtherVideo).not.toHaveBeenCalled();
});
