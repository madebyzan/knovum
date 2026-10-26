import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";
import { nextTick, reactive } from "vue";

import type { RunObservation } from "../../contracts/generated/RunObservation";
import type { startDerivationAction } from "../start-action";

const space = vi.hoisted(() => ({
  state: undefined as unknown as { activeSpaceId: string; nodes: { id: string }[] },
}));
const start = vi.hoisted(() => vi.fn<typeof startDerivationAction>());
vi.mock("../../session/space-store", () => ({ useSpaceStore: () => space.state }));
vi.mock("../start-action", () => ({ startDerivationAction: start }));
vi.mock("../../../../foundation/i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));

import { canGenerate, generatorFrame, uploadPosition } from "../generators/canvas-generator";
import { useCanvasGeneratorStore } from "../generators/generator-store";

const TARGET = { type: "agent", runtimeKind: "codex" } as const;
const SOURCE = {
  nodeId: "photo",
  title: "Photo",
  sourceRevision: "asset:1",
  revisionBasis: "asset",
} as const;

function run(state: RunObservation["state"] = "queued") {
  return { id: "run", state } as RunObservation;
}

function ready(kind: "image" | "video" | "document" = "image") {
  const store = useCanvasGeneratorStore();
  store.open(kind, { center: { x: 0, y: 0 } });
  const [generator] = store.generators;
  store.update(generator!.id, { prompt: "A harbour at dawn\nwatercolour", target: TARGET });
  return { store, id: generator!.id };
}

beforeEach(() => {
  space.state = reactive({ activeSpaceId: "space", nodes: [{ id: "photo" }, { id: "notes" }] });
  setActivePinia(createPinia());
  start.mockReset();
});

it("opens where the person is looking, or beside what they selected with it as reference", () => {
  const store = useCanvasGeneratorStore();
  store.open("image", { center: { x: 0, y: 0 } });
  const [image] = store.generators;
  expect(generatorFrame(image!)).toEqual({ x: -180, y: -180, width: 360, height: 360 });
  store.update(image!.id, { prompt: "kept" });
  store.open("video", { beside: { x: 0, y: 0, width: 200, height: 100 } }, ["photo", "photo"]);
  const video = store.generators[1]!;
  expect(generatorFrame(video)).toEqual({ x: 248, y: 0, width: 360, height: 202.5 });
  expect(video.references).toEqual(["photo"]);
  expect(store.active?.id).toBe(video.id);
  expect(store.takePromptFocus(video.id)).toBe(true);
  expect(store.takePromptFocus(video.id)).toBe(false);
});

it("asks for the output in its own frame, named by the prompt, from the references as pinned", async () => {
  start.mockResolvedValue(run());
  const { store, id } = ready();
  store.addReferences(id, ["photo", "photo"]);
  expect(store.generators[0]!.references).toEqual(["photo"]);
  await store.generate(id, [SOURCE]);
  expect(start).toHaveBeenCalledWith(
    expect.objectContaining({
      actionId: "generate-visualization",
      sources: [SOURCE],
      parameters: {
        title: "A harbour at dawn",
        intent: "A harbour at dawn\nwatercolour",
        aspectRatio: "1:1",
      },
      target: TARGET,
      outputFrame: { x: -180, y: -180, width: 360, height: 360 },
    }),
  );
  expect(store.active).toBeUndefined();
  expect(store.generators[0]!.submission).toEqual({ runId: "run" });
});

it("steps aside once the output it asked for stands in its frame", async () => {
  start.mockResolvedValue(run());
  const { store, id } = ready();
  await store.generate(id, []);
  space.state.nodes = [...space.state.nodes, { id: "run-output" }];
  await nextTick();
  expect(store.generators).toEqual([]);
});

it("gives the frame back with its prompt when the Run ends without publishing", async () => {
  start.mockImplementation(async ({ onUpdate }) => {
    queueMicrotask(() => onUpdate?.(run("failed")));
    return run();
  });
  const { store, id } = ready();
  await store.generate(id, []);
  await Promise.resolve();
  expect(store.generators[0]).toMatchObject({
    id,
    prompt: "A harbour at dawn\nwatercolour",
    submission: undefined,
  });
});

it("asks for nothing without a prompt and an executor", async () => {
  const store = useCanvasGeneratorStore();
  store.open("image", { center: { x: 0, y: 0 } });
  await store.generate(store.generators[0]!.id, []);
  expect(start).not.toHaveBeenCalled();
});

it("leaves on a press anywhere but a generator, and an empty one leaves nothing behind", () => {
  const { store, id } = ready();
  const frame = document.createElement("div");
  frame.dataset.canvasGenerator = "";
  store.release(frame.appendChild(document.createElement("span")));
  expect(store.active?.id).toBe(id);
  store.release(document.body);
  expect(store.active).toBeUndefined();
  expect(store.generators.map((generator) => generator.id)).toEqual([id]);

  store.open("video", { center: { x: 0, y: 0 } });
  store.release(document.body);
  expect(store.generators.map((generator) => generator.id)).toEqual([id]);
});

it("keeps its composer while picking references from the canvas", () => {
  const { store, id } = ready();
  store.startPicking();
  store.release(document.body);
  expect(store.active?.id).toBe(id);
  expect(store.picking).toBe(true);
  store.deactivate();
  expect(store.picking).toBe(false);
});

it("lets a deleted node go from its references, and is removed when discarded", async () => {
  const { store, id } = ready();
  store.addReferences(id, ["photo", "notes"]);
  space.state.nodes = [{ id: "notes" }];
  await nextTick();
  expect(store.generators[0]!.references).toEqual(["notes"]);
  store.discard(id);
  expect(store.generators).toEqual([]);
  expect(store.active).toBeUndefined();
});

it("does not mistake a Space that is still loading for deleted references", async () => {
  const { store, id } = ready();
  store.addReferences(id, ["photo"]);
  space.state.activeSpaceId = "other";
  space.state.nodes = [];
  await nextTick();
  space.state.activeSpaceId = "space";
  space.state.nodes = [{ id: "photo" }];
  await nextTick();
  expect(store.generators[0]!.references).toEqual(["photo"]);
});

it("generates once it is asked for something by someone, and uploads land to its left", () => {
  const { store, id } = ready();
  const generator = store.generators[0]!;
  expect(canGenerate(generator)).toBe(true);
  expect(canGenerate({ ...generator, prompt: "  " })).toBe(false);
  expect(canGenerate({ ...generator, target: undefined })).toBe(false);
  expect(canGenerate({ ...generator, submission: "starting" })).toBe(false);
  expect(uploadPosition(generator)).toEqual({ x: -180 - 48 - 360, y: -180 });
  expect(store.generators[0]!.id).toBe(id);
});
