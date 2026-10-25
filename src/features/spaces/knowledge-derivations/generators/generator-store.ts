import { defineStore } from "pinia";
import { computed, shallowRef, watch } from "vue";

import { i18n } from "../../../../foundation/i18n/instance";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import type { CanvasPoint, CanvasRect } from "../../canvas/contracts/canvas-editor-port";
import type { OutputFrame } from "../../contracts/generated/OutputFrame";
import type { RunState } from "../../contracts/generated/RunState";
import { runOutputNodeId } from "../../runs/run-output";
import { useSpaceStore } from "../../session/space-store";
import { MAX_CREATION_SOURCES, type CreationSource } from "../catalog/creation-source";
import { startDerivationAction } from "../start-action";
import {
  GENERATOR_KINDS,
  generatorFrameSize,
  generatorTitle,
  type GeneratorKind,
  type GeneratorRequest,
} from "./generator-kinds";

/**
 * A frame on the canvas where something is about to be made: what to make, from what, by whom.
 * It is a draft of the session, not a node — nothing is stored until the person asks for it, and
 * then the output is reserved in this frame and takes its place.
 */
export interface CanvasGenerator extends GeneratorRequest {
  id: string;
  spaceId: string;
  kind: GeneratorKind;
  x: number;
  y: number;
  references: readonly CreationSource[];
  target: TaskTarget | undefined;
  /** The Run it asked for, until that Run's output takes its place; "starting" until accepted. */
  submission: "starting" | { runId: string } | undefined;
}

export type GeneratorPlacement = { center: CanvasPoint } | { beside: CanvasRect };

type GeneratorChanges = Partial<
  Pick<CanvasGenerator, "prompt" | "aspectRatio" | "durationSeconds" | "target">
>;

/** Room between a selection and the generator opened beside it, in canvas units. */
const BESIDE_GAP = 48;
/** A Run that ends in one of these states never publishes into the frame. */
const UNPUBLISHED: ReadonlySet<RunState> = new Set(["failed", "completed_with_errors", "cancelled"]);

export function generatorFrame({ x, y, aspectRatio }: CanvasGenerator): OutputFrame {
  return { x, y, ...generatorFrameSize(aspectRatio) };
}

export const useCanvasGeneratorStore = defineStore("spaces-canvas-generators", () => {
  const space = useSpaceStore();
  const all = shallowRef<readonly CanvasGenerator[]>([]);
  const activeId = shallowRef<string>();
  const generators = computed(() => all.value.filter(({ spaceId }) => spaceId === space.activeSpaceId));
  const active = computed(() => generators.value.find(({ id }) => id === activeId.value));

  function find(id: string) {
    return all.value.find((generator) => generator.id === id);
  }

  function patch(id: string, changes: Partial<CanvasGenerator>) {
    all.value = all.value.map((generator) =>
      generator.id === id ? { ...generator, ...changes } : generator,
    );
  }

  function open(
    kind: GeneratorKind,
    placement: GeneratorPlacement,
    references: readonly CreationSource[] = [],
  ) {
    const spaceId = space.activeSpaceId;
    if (!spaceId) return;
    const [aspectRatio = "1:1"] = GENERATOR_KINDS[kind].aspectRatios;
    const { width, height } = generatorFrameSize(aspectRatio);
    const origin =
      "center" in placement
        ? { x: placement.center.x - width / 2, y: placement.center.y - height / 2 }
        : { x: placement.beside.x + placement.beside.width + BESIDE_GAP, y: placement.beside.y };
    const generator: CanvasGenerator = {
      id: crypto.randomUUID(),
      spaceId,
      kind,
      ...origin,
      prompt: "",
      aspectRatio,
      durationSeconds: 5,
      references: references.slice(0, MAX_CREATION_SOURCES),
      target: undefined,
      submission: undefined,
    };
    all.value = [...all.value, generator];
    activeId.value = generator.id;
  }

  function activate(id: string) {
    activeId.value = id;
  }

  /** Puts the composer away; the frame and its draft stay. */
  function deactivate() {
    activeId.value = undefined;
  }

  /** A press anywhere on the canvas but a generator puts the composer away. */
  function release(target: EventTarget | null) {
    if (!(target instanceof Element && target.closest("[data-canvas-generator]"))) deactivate();
  }

  function update(id: string, changes: GeneratorChanges) {
    patch(id, changes);
  }

  function move(id: string, x: number, y: number) {
    patch(id, { x, y });
  }

  function addReferences(id: string, sources: readonly CreationSource[]) {
    const generator = find(id);
    if (!generator) return;
    const references = new Map(
      [...generator.references, ...sources].map((source) => [source.nodeId, source]),
    );
    patch(id, { references: [...references.values()].slice(0, MAX_CREATION_SOURCES) });
  }

  function removeReference(id: string, nodeId: string) {
    const generator = find(id);
    if (generator)
      patch(id, { references: generator.references.filter((source) => source.nodeId !== nodeId) });
  }

  function close(id: string) {
    all.value = all.value.filter((generator) => generator.id !== id);
    if (activeId.value === id) deactivate();
  }

  async function generate(id: string) {
    const generator = find(id);
    if (!generator || generator.submission || !generator.target || !generator.prompt.trim()) return;
    const kind = GENERATOR_KINDS[generator.kind];
    patch(id, { submission: "starting" });
    deactivate();
    const run = await startDerivationAction({
      actionId: kind.actionId,
      sources: generator.references,
      parameters: kind.parameters(generatorTitle(generator.prompt), generator),
      title: i18n.global.t(`spaces.creation.generator.task.${generator.kind}`),
      target: generator.target,
      outputFrame: generatorFrame(generator),
      onUpdate: ({ state }) => {
        if (UNPUBLISHED.has(state)) reopen(id);
      },
    });
    if (find(id)?.submission === "starting") patch(id, { submission: run ? { runId: run.id } : undefined });
  }

  /** The Run ended without publishing: the frame is the person's again, with what they asked. */
  function reopen(id: string) {
    if (find(id)) patch(id, { submission: undefined });
  }

  // A generator is done once the output it asked for stands in its frame.
  watch(
    () => space.nodes,
    (nodes) => {
      const ids = new Set(nodes.map(({ id }) => id));
      const published = all.value.filter(
        ({ submission }) => typeof submission === "object" && ids.has(runOutputNodeId(submission.runId)),
      );
      for (const { id } of published) close(id);
    },
  );
  watch(() => space.activeSpaceId, deactivate);

  return {
    generators,
    active,
    open,
    activate,
    deactivate,
    release,
    update,
    move,
    addReferences,
    removeReference,
    close,
    generate,
  };
});
