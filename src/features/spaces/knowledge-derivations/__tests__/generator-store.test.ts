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

import { generatorFrame, useCanvasGeneratorStore } from "../generators/generator-store";

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
  space.state = reactive({ activeSpaceId: "space", nodes: [] });
  setActivePinia(createPinia());
  start.mockReset();
});

it("opens where the person is looking, or beside what they selected with it as reference", () => {
  const store = useCanvasGeneratorStore();
  store.open("image", { center: { x: 100, y: 100 } });
  expect(generatorFrame(store.generators[0]!)).toEqual({ x: -80, y: -80, width: 360, height: 360 });
  store.open("video", { beside: { x: 0, y: 0, width: 200, height: 100 } }, [SOURCE]);
  const video = store.generators[1]!;
  expect(generatorFrame(video)).toEqual({ x: 248, y: 0, width: 360, height: 202.5 });
  expect(video.references).toEqual([SOURCE]);
  expect(store.active?.id).toBe(video.id);
});

it("asks for the output in its own frame, named by the prompt, from whatever it references", async () => {
  start.mockResolvedValue(run());
  const { store, id } = ready();
  store.addReferences(id, [SOURCE, SOURCE]);
  await store.generate(id);
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

it("gives way to the output once it stands in the frame", async () => {
  start.mockResolvedValue(run());
  const { store, id } = ready("document");
  await store.generate(id);
  space.state.nodes = [{ id: "run-output" }];
  await nextTick();
  expect(store.generators).toEqual([]);
});

it("gives the frame back with its prompt when the Run ends without publishing", async () => {
  start.mockImplementation(async ({ onUpdate }) => {
    queueMicrotask(() => onUpdate?.(run("failed")));
    return run();
  });
  const { store, id } = ready();
  await store.generate(id);
  await Promise.resolve();
  expect(store.generators[0]).toMatchObject({
    id,
    submission: undefined,
    prompt: "A harbour at dawn\nwatercolour",
  });
});

it("does not ask without a prompt and an executor", async () => {
  const store = useCanvasGeneratorStore();
  store.open("image", { center: { x: 0, y: 0 } });
  await store.generate(store.generators[0]!.id);
  expect(start).not.toHaveBeenCalled();
});

it("puts the composer away on a press anywhere but a generator", () => {
  const { store } = ready();
  const frame = document.createElement("div");
  frame.dataset.canvasGenerator = "";
  const inside = frame.appendChild(document.createElement("span"));
  store.release(inside);
  expect(store.active).toBeDefined();
  store.release(document.body);
  expect(store.active).toBeUndefined();
});
