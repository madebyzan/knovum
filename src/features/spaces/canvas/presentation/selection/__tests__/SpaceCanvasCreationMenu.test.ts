import { mount } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { afterEach, describe, expect, it } from "vitest";
import { defineComponent, h } from "vue";

import { i18n } from "../../../../../../foundation/i18n/instance";
import { createSpaceNode } from "../../../../canvas/node";
import { creationIntentCatalog } from "../../../../knowledge-derivations/catalog/creation-intent-catalog";
import type { CreationIntentDecisions } from "../../../../knowledge-derivations/catalog/creation-intent-definitions";
import {
  hasVisibleCreationActions,
  visibleCreationActions,
} from "../../../../knowledge-derivations/presentation/visible-derivation-actions";
import SpaceCanvasCreationMenu from "../SpaceCanvasCreationMenu.vue";

function makeDecisions(overrides: Partial<CreationIntentDecisions> = {}): CreationIntentDecisions {
  return {
    "draft-document": { state: "hidden" },
    "generate-visualization": { state: "hidden" },
    "generate-video": { state: "hidden" },
    "remove-image-background": { state: "hidden" },
    "split-image-layers": { state: "hidden" },
    "slice-sticker-sheet": { state: "hidden" },
    "add-video-subtitles": { state: "hidden" },
    ...overrides,
  };
}

describe("SpaceCanvasCreationMenu", () => {
  let wrapper: ReturnType<typeof mount> | undefined;

  afterEach(() => {
    wrapper?.unmount();
    document.body.innerHTML = "";
  });

  it("includes only generative creation intents and excludes resource utilities such as sticker slicing", () => {
    const decisions = makeDecisions({
      "draft-document": { state: "ready" },
      "generate-visualization": { state: "ready" },
      "generate-video": { state: "blocked", reason: "too-many-video-images" },
      "remove-image-background": { state: "ready" },
      "split-image-layers": { state: "ready" },
      "slice-sticker-sheet": { state: "ready" },
      "add-video-subtitles": { state: "ready" },
    });

    expect(visibleCreationActions(decisions).map((action) => action.id)).toEqual([
      "draft-document",
      "generate-visualization",
      "generate-video",
    ]);
    expect(hasVisibleCreationActions(makeDecisions())).toBe(false);
    expect(
      hasVisibleCreationActions(
        makeDecisions({
          "remove-image-background": { state: "ready" },
          "slice-sticker-sheet": { state: "ready" },
        }),
      ),
    ).toBe(false);
  });

  it("hides creation actions for non-semantic utility nodes such as credentials and portals", () => {
    const credential = createSpaceNode(
      {
        kind: "credential",
        title: "OpenAI",
        credentialKind: "api-key",
        credentialConfigured: true,
      },
      { x: 0, y: 0 },
      "cred-1",
    );
    const portal = createSpaceNode(
      {
        kind: "portal",
        title: "Jump",
        targetSpaceId: "space-2",
      },
      { x: 0, y: 0 },
      "portal-1",
    );

    expect(hasVisibleCreationActions(creationIntentCatalog([credential]).decisions)).toBe(false);
    expect(hasVisibleCreationActions(creationIntentCatalog([credential, portal]).decisions)).toBe(false);
  });

  it("renders a primary trigger button only when visible creation actions exist", () => {
    const Host = defineComponent({
      props: { derivations: { type: Object, required: true } },
      setup(props) {
        return () =>
          h(ToolbarRoot, null, () =>
            h(SpaceCanvasCreationMenu, {
              derivations: props.derivations as CreationIntentDecisions,
            }),
          );
      },
    });

    wrapper = mount(Host, {
      attachTo: document.body,
      global: { plugins: [i18n] },
      props: { derivations: makeDecisions() },
    });
    expect(wrapper.find("button").exists()).toBe(false);
    wrapper.unmount();

    wrapper = mount(Host, {
      attachTo: document.body,
      global: { plugins: [i18n] },
      props: {
        derivations: makeDecisions({
          "draft-document": { state: "ready" },
          "generate-visualization": { state: "ready" },
          "generate-video": { state: "ready" },
        }),
      },
    });
    const button = wrapper.get("button");
    expect(button.text()).toContain("Create");
    expect(button.attributes("aria-label")).toBe("Create from selection");
  });
});
