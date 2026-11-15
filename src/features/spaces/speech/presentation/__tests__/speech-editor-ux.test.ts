import { mount } from "@vue/test-utils";
import { TabsRoot } from "reka-ui";
import { expect, it, onTestFinished } from "vitest";
import { defineComponent, shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { VideoDubbingOutputConfig } from "../../../contracts/generated/VideoDubbingOutputConfig";
import SpeechDeliveryInstructionField from "../SpeechDeliveryInstructionField.vue";
import SpeechSegmentedTabs from "../SpeechSegmentedTabs.vue";
import VideoDubbingLineEditor from "../VideoDubbingLineEditor.vue";
import VideoDubbingMixSettings from "../VideoDubbingMixSettings.vue";

it("keeps empty collapsible delivery instructions compact until expanded and shows 500-char counter", async () => {
  const value = shallowRef("");
  const Host = defineComponent({
    components: { SpeechDeliveryInstructionField },
    setup: () => ({ value }),
    template: '<SpeechDeliveryInstructionField v-model="value" collapsible />',
  });
  const wrapper = mount(Host, { global: { plugins: [i18n] } });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.find("textarea").exists()).toBe(false);
  await wrapper.get("button").trigger("click");
  expect(wrapper.find("textarea").exists()).toBe(true);
  await wrapper.get("textarea").setValue("Warm");
  expect(wrapper.text()).toContain("4/500");
  await wrapper.setProps({});
  value.value = "Calm pace";
  await wrapper.vm.$nextTick();
  expect(wrapper.text()).toContain("9/500");
});

it("renders timed cue metadata and emits source-window & preview events from VideoDubbingLineEditor", async () => {
  const wrapper = mount(VideoDubbingLineEditor, {
    props: {
      index: 0,
      modelValue: {
        key: "line-1",
        startSeconds: "1.0",
        endSeconds: "4.5",
        text: "Hello Knovum",
        speakerKey: "Host",
        voiceRevisionId: "voice-1",
        deliveryInstruction: "Crisp",
        sourceText: "你好 Knovum",
      },
      voiceOptions: [{ value: "voice-1", label: "Adam · v1" }],
      deliveryInstructionVoiceIds: new Set(["voice-1"]),
      lexiconValid: true,
      showSource: true,
      disabled: false,
    },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.text()).toContain("#1");
  expect(wrapper.text()).toContain("3.5s");
  expect(wrapper.text()).toContain("你好 Knovum");
  const sourceBtn = wrapper.get(
    `button[aria-label="${i18n.global.t("spaces.speech.videoDubbing.auditionSourceWindow")}"]`,
  );
  await sourceBtn.trigger("click");
  expect(wrapper.emitted("source")).toEqual([[1000, 4500]]);
  const previewBtn = wrapper.get(`button[aria-label="${i18n.global.t("spaces.speech.preview.action")}"]`);
  await previewBtn.trigger("click");
  expect(wrapper.emitted("preview")).toEqual([["Hello Knovum", "voice-1", "Crisp"]]);
});

it("updates background gain presets and retains Mix checkboxes in VideoDubbingMixSettings", async () => {
  const config = shallowRef<VideoDubbingOutputConfig>({
    format: "mp4",
    video: { assetPath: "/clip.mp4", sourceRevision: "1", durationMs: 8000 },
    background: { assetPath: "/bgm.wav", sourceRevision: "1", durationMs: 8000 },
    backgroundSeparation: null,
    voiceGainDb: 0,
    backgroundGainDb: 0,
    translation: null,
    preserveOriginalOutsideDialogue: false,
    sourceDialogueIntervals: [],
    retainOriginalTrack: true,
    autoBackgroundSeparationModel: "htDemucs",
  });
  const voiceGain = shallowRef("0");
  const backgroundGain = shallowRef("0");
  const Host = defineComponent({
    components: { VideoDubbingMixSettings },
    setup: () => ({ config, voiceGain, backgroundGain }),
    template:
      '<VideoDubbingMixSettings v-model="config" v-model:voice-gain="voiceGain" v-model:background-gain="backgroundGain" space-id="s1" :disabled="false" />',
  });
  const wrapper = mount(Host, {
    global: {
      plugins: [i18n],
      stubs: { VideoBackgroundSelect: { template: '<div data-testid="bg-select" />' } },
    },
  });
  onTestFinished(() => wrapper.unmount());
  const softBtn = wrapper
    .findAll("button")
    .find((b) => b.text() === i18n.global.t("spaces.speech.videoDubbing.backgroundPresetSoft"));
  await softBtn!.trigger("click");
  expect(backgroundGain.value).toBe("-6");
  const sliders = wrapper.findAll('[role="slider"]');
  expect(sliders).toHaveLength(2);
});

it("displays optional tab badges and validation indicators in SpeechSegmentedTabs", () => {
  const wrapper = mount(
    defineComponent({
      components: { TabsRoot, SpeechSegmentedTabs },
      template: `
        <TabsRoot model-value="script">
          <SpeechSegmentedTabs label="Pages" :items="[
            { value: 'script', label: 'Script', badge: 4 },
            { value: 'mixing', label: 'Mix', invalid: true },
          ]" />
        </TabsRoot>
      `,
    }),
  );
  onTestFinished(() => wrapper.unmount());
  const tabs = wrapper.findAll('[role="tab"]');
  expect(tabs[0]!.text()).toContain("4");
  expect(tabs[1]!.find(".bg-surface-danger").exists()).toBe(true);
});

it("keeps VideoDubbingLineEditor compact controls free of overflow wrapping classes", () => {
  const wrapper = mount(VideoDubbingLineEditor, {
    props: {
      index: 1,
      modelValue: {
        key: "line-2",
        startSeconds: "0.0",
        endSeconds: "2.0",
        text: "Short cue",
        speakerKey: "Narrator",
        voiceRevisionId: "voice-1",
        deliveryInstruction: "",
        sourceText: "",
      },
      voiceOptions: [{ value: "voice-1", label: "Adam · v1" }],
      deliveryInstructionVoiceIds: new Set<string>(),
      lexiconValid: true,
      showSource: false,
      disabled: false,
    },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  const html = wrapper.html();
  expect(html).toContain("appearance-none");
  expect(html).toContain("border-transparent bg-transparent");
});
