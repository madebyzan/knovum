import { shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, reactive, shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import type { ImageOperationCapabilities } from "../contracts/generated/ImageOperationCapabilities";
import type { CreationIntentId } from "../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import {
  blockedImageCapabilities,
  imageSourceNode,
  selectedImageIntents,
} from "./support/image-operation-fixtures";

const context = vi.hoisted(() => ({
  space: null as null | { activeSpaceId: string; nodes: SpaceCanvasNode[] },
  capabilities: null as null | {
    data: ReturnType<typeof shallowRef<ImageOperationCapabilities>>;
    isFetching: ReturnType<typeof shallowRef<boolean>>;
    isError: ReturnType<typeof shallowRef<boolean>>;
  },
  allowed: true,
  derive: vi.fn<(...args: unknown[]) => Promise<void>>(),
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => context.space! }));
vi.mock("../canvas/use-node-operation-gate", () => ({
  useNodeOperationGate: () => ({ allows: () => context.allowed }),
}));
vi.mock("../knowledge-derivations/image-operation", () => ({ deriveImage: context.derive }));
vi.mock("../knowledge-derivations/use-image-capabilities", () => ({
  useImageOperationCapabilities: () => context.capabilities!,
}));
vi.mock("../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("vue-router", () => ({ useRouter: () => ({ push: vi.fn<() => Promise<void>>() }) }));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));
import { useImageNodeBarState } from "../canvas/presentation/images/use-image-node-bar-state";
import SpaceCanvasObjectRemoval from "../image-editing/presentation/SpaceCanvasObjectRemoval.vue";
import SpaceImageObjectRemovalControls from "../image-editing/presentation/SpaceImageObjectRemovalControls.vue";
import { useImageRemovalSessionStore } from "../knowledge-derivations/image-removal-session-store";

let node: SpaceCanvasNode<SpaceImagePayload>;
beforeEach(() => {
  setActivePinia(createPinia());
  context.allowed = true;
  context.derive.mockReset().mockResolvedValue(undefined);
  node = imageSourceNode();
  context.space = reactive({ activeSpaceId: "space", nodes: [node] });
  context.capabilities = {
    data: shallowRef(blockedImageCapabilities()),
    isFetching: shallowRef(false),
    isError: shallowRef(false),
  };
});

function openTool() {
  let bar!: ReturnType<typeof useImageNodeBarState>;
  const wrapper = shallowMount({
    setup() {
      bar = useImageNodeBarState(
        shallowRef(node),
        shallowRef(selectedImageIntents()),
        vi.fn<(actionId: CreationIntentId) => void>(),
      );
      return () => null;
    },
  });
  bar.beginRemoval();
  return { bar, wrapper };
}
function setCapabilities(patch: Partial<ImageOperationCapabilities>) {
  const current = context.capabilities!.data.value!;
  context.capabilities!.data.value = {
    ...current,
    ...patch,
    targets: current.targets.map((target) =>
      Object.assign({}, target, {
        availability: patch.maskedRemoval ?? target.availability,
      }),
    ),
  };
}
function openPanel() {
  const removal = useImageRemovalSessionStore();
  removal.begin(node.id);
  const wrapper = shallowMount(SpaceCanvasObjectRemoval, { props: { node } });
  return { removal, wrapper, panel: wrapper.findComponent(SpaceImageObjectRemovalControls) };
}

describe("local brush repair and remote described removal", () => {
  it("admits the tool without a model and permits painting without submitting", () => {
    const { wrapper } = openTool();
    const removal = useImageRemovalSessionStore();
    removal.addStroke([0.2, 0.3, 0.4, 0.5]);
    expect(context.derive).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("still rejects tool entry while another operation owns the node", () => {
    context.allowed = false;
    const { wrapper } = openTool();
    expect(useImageRemovalSessionStore().active).toBeUndefined();
    wrapper.unmount();
  });

  it("blocks confirmation and its handler without configuration, then enables the same draft", async () => {
    const { removal, wrapper, panel } = openPanel();
    removal.addStroke([0.2, 0.3, 0.4, 0.5]);
    await nextTick();
    expect(panel.props("canGenerate")).toBe(false);
    expect(panel.props("help")?.label).toContain("modelUnavailable");
    panel.vm.$emit("generate");
    expect(context.derive).not.toHaveBeenCalled();
    setCapabilities({ maskedRemoval: { state: "ready" } });
    await nextTick();
    expect(panel.props("canGenerate")).toBe(true);
    panel.vm.$emit("generate");
    expect(context.derive).toHaveBeenCalledOnce();
    expect(context.derive.mock.calls[0]?.[1]).toEqual({
      kind: "remove-object",
      strokes: [{ diameter: 0.045, points: [0.2, 0.3, 0.4, 0.5] }],
    });
    wrapper.unmount();
  });

  it("keeps painting available while capability checking or a failed check blocks generation", async () => {
    const { removal, wrapper, panel } = openPanel();
    removal.addStroke([0.2, 0.3, 0.4, 0.5]);
    context.capabilities!.isFetching.value = true;
    await nextTick();
    expect(removal.canDraw).toBe(true);
    expect(panel.props("canGenerate")).toBe(false);
    context.capabilities!.isFetching.value = false;
    context.capabilities!.isError.value = true;
    await nextTick();
    panel.vm.$emit("generate");
    expect(context.derive).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("uses description removal only when explicitly selected, preserving the painted draft", async () => {
    const { removal, wrapper, panel } = openPanel();
    removal.addStroke([0.2, 0.3, 0.4, 0.5]);
    setCapabilities({
      maskedRemoval: { state: "blocked", reason: "provider_unsupported" },
      promptRemoval: { state: "unverified" },
    });
    await nextTick();
    expect(panel.props("canGenerate")).toBe(false);
    expect(removal.active?.mode).toBe("mask");
    panel.vm.$emit("updateMode", "prompt");
    panel.vm.$emit("updatePrompt", "  remove the sign  ");
    await nextTick();
    expect(removal.canDraw).toBe(false);
    expect(panel.props("canGenerate")).toBe(true);
    expect(removal.active?.strokes).toHaveLength(1);
    panel.vm.$emit("generate");
    expect(context.derive).toHaveBeenCalledOnce();
    expect(context.derive.mock.calls[0]?.[1]).toEqual({
      kind: "remove-object-prompt",
      prompt: "remove the sign",
    });
    wrapper.unmount();
  });

  it("does not submit an empty description or a closed selection", async () => {
    const { removal, wrapper, panel } = openPanel();
    removal.setMode("prompt");
    setCapabilities({ promptRemoval: { state: "ready" } });
    await nextTick();
    expect(panel.props("canGenerate")).toBe(false);
    removal.setPrompt("remove the sign");
    removal.cancel();
    panel.vm.$emit("generate");
    expect(context.derive).not.toHaveBeenCalled();
    wrapper.unmount();
  });
});
