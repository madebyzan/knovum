import { afterEach, expect, it } from "vitest";
import { computed } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import { imageActionHelp } from "../canvas/presentation/images/image-action-help";
import { videoEditCommands } from "../commands/video-edit-commands";
import { captureActionLabel } from "../workflows/presentation/workflow-capture-labels";

const originalLocale = i18n.global.locale.value;
afterEach(() => {
  i18n.global.locale.value = originalLocale;
});

it("updates consumer messages after a language switch without a supplied translator", () => {
  i18n.global.locale.value = "en";
  const undo = computed(() => videoEditCommands().find(({ id }) => id === "video-edit.undo")?.title);
  const help = computed(() => imageActionHelp({ state: "checking" })?.label);
  const capture = computed(() => captureActionLabel("draft-document"));
  expect(undo.value).toBe("Undo");
  expect(help.value).toBe("Checking image capabilities…");
  const englishCapture = capture.value;

  i18n.global.locale.value = "zh-CN";
  expect(undo.value).toBe("撤销");
  expect(help.value).toBe("正在检查图像处理能力…");
  expect(capture.value).not.toBe(englishCapture);
  expect(captureActionLabel("custom-action")).toBe("custom-action");
});
