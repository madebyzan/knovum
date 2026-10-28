import { mount } from "@vue/test-utils";
import { describe, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, nextTick } from "vue";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { useSpeechProjectLeave } from "../speech/use-speech-project-leave";
import { flushSpaceSessionContent } from "../workspace/session-participants";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

describe("editor leave protection", () => {
  it("does not switch speech projects when the draft leave prompt is cancelled", async () => {
    const openProject = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    let speech: ReturnType<typeof useSpeechProjectLeave> | undefined;
    const Harness = defineComponent({
      setup() {
        speech = useSpeechProjectLeave(
          vi.fn<() => Promise<boolean>>().mockResolvedValue(true),
          openProject,
          vi.fn<() => void>(),
          vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
        );
        speech.updateDraft(true, {
          sourceRevision: "revision",
          pronunciationLexicon: {},
          outputConfig: {},
          segments: [],
        });
        return () => null;
      },
    });
    const wrapper = mount(Harness);
    onTestFinished(() => wrapper.unmount());

    const switching = speech?.selectProject("next-project");
    await nextTick();
    expect(speech?.leave.dialogOpen.value).toBe(true);
    speech?.leave.cancel();
    await switching;

    expect(openProject).not.toHaveBeenCalled();
  });

  it.each([true, false])(
    "settles an admitted speech save before evaluating leave (%s)",
    async (succeeded) => {
      let finishSave!: () => void;
      const admittedSave = new Promise<void>((resolve) => (finishSave = resolve));
      const discard = vi.fn<() => void>();
      let speech!: ReturnType<typeof useSpeechProjectLeave>;
      const wrapper = mount(
        defineComponent({
          setup() {
            speech = useSpeechProjectLeave(
              vi.fn<() => Promise<boolean>>().mockResolvedValue(true),
              vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
              discard,
              () => admittedSave,
            );
            speech.updateDraft(true, {
              sourceRevision: "revision",
              pronunciationLexicon: {},
              outputConfig: {},
              segments: [],
            });
            return () => null;
          },
        }),
      );
      onTestFinished(() => wrapper.unmount());
      const leaving = flushSpaceSessionContent();
      await nextTick();
      expect(speech.leave.dialogOpen.value).toBe(false);
      if (succeeded) speech.updateDraft(false);
      finishSave();
      await vi.waitFor(() => expect(speech.leave.dialogOpen.value).toBe(!succeeded));
      if (!succeeded) speech.leave.cancel();
      const outcome = await leaving.then(
        () => undefined,
        (error: unknown) => error,
      );
      const expectedOutcome = succeeded ? undefined : new ApplicationLeaveCancelled();
      expect(outcome).toEqual(expectedOutcome);
      expect(speech.draft.value?.sourceRevision).toBe(succeeded ? undefined : "revision");
      expect(discard).not.toHaveBeenCalled();
    },
  );
});
