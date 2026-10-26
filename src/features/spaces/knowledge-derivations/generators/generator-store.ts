import { defineStore } from "pinia";
import { computed, shallowRef, watch } from "vue";

import { i18n } from "../../../../foundation/i18n/instance";
import type { RunState } from "../../contracts/generated/RunState";
import { runOutputNodeId } from "../../runs/run-output";
import { useSpaceStore } from "../../session/space-store";
import { MAX_CREATION_SOURCES, type CreationSource } from "../catalog/creation-source";
import { startDerivationAction } from "../start-action";
import {
  generatorFrame,
  generatorOrigin,
  pristine,
  type CanvasGenerator,
  type GeneratorChanges,
  type GeneratorPlacement,
} from "./canvas-generator";
import { GENERATOR_KINDS, generatorFrameSize, generatorTitle, type GeneratorKind } from "./generator-kinds";

/** A Run that ends in one of these states never publishes into the frame. */
const UNPUBLISHED: ReadonlySet<RunState> = new Set(["failed", "completed_with_errors", "cancelled"]);

export const useCanvasGeneratorStore = defineStore("spaces-canvas-generators", () => {
  const space = useSpaceStore();
  const all = shallowRef<readonly CanvasGenerator[]>([]);
  const activeId = shallowRef<string>();
  /** The active generator is taking its references from what the person clicks on the canvas. */
  const picking = shallowRef(false);
  /**
   * The generator whose prompt takes focus when its composer appears: one just opened is there to
   * be written to, while pressing an existing frame selects it as an object, the way pressing a
   * node does — so Delete removes it rather than editing its prompt.
   */
  const promptFocus = shallowRef<string>();
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

  function open(kind: GeneratorKind, placement: GeneratorPlacement, references: readonly string[] = []) {
    const spaceId = space.activeSpaceId;
    if (!spaceId) return;
    deactivate();
    const [aspectRatio = "1:1"] = GENERATOR_KINDS[kind].aspectRatios;
    const origin = generatorOrigin(placement, generatorFrameSize(aspectRatio));
    const generator: CanvasGenerator = {
      id: crypto.randomUUID(),
      spaceId,
      kind,
      ...origin,
      prompt: "",
      aspectRatio,
      durationSeconds: 5,
      references: [...new Set(references)].slice(0, MAX_CREATION_SOURCES),
      target: undefined,
      submission: undefined,
    };
    all.value = [...all.value, generator];
    activeId.value = generator.id;
    promptFocus.value = generator.id;
  }

  function activate(id: string) {
    if (activeId.value !== id) deactivate();
    activeId.value = id;
  }

  /** The person has left the generator: its composer goes away, and an empty one goes with it. */
  function deactivate() {
    const current = activeId.value && find(activeId.value);
    picking.value = false;
    activeId.value = undefined;
    if (current && pristine(current)) discard(current.id);
  }

  /** A press anywhere on the canvas but a generator leaves it — unless it is a press picking a reference. */
  function release(target: EventTarget | null) {
    if (picking.value) return;
    if (!(target instanceof Element && target.closest("[data-canvas-generator]"))) deactivate();
  }

  /** True once, for the composer that should focus its prompt. */
  function takePromptFocus(id: string) {
    const requested = promptFocus.value === id;
    promptFocus.value = undefined;
    return requested;
  }

  function startPicking() {
    if (active.value) picking.value = true;
  }

  function stopPicking() {
    picking.value = false;
  }

  function update(id: string, changes: GeneratorChanges) {
    patch(id, changes);
  }

  function move(id: string, x: number, y: number) {
    patch(id, { x, y });
  }

  /** Adds what is not referenced yet, in order, up to what one Run can pin; returns what was added. */
  function addReferences(id: string, nodeIds: readonly string[]) {
    const generator = find(id);
    if (!generator) return [];
    const references = [...new Set([...generator.references, ...nodeIds])].slice(0, MAX_CREATION_SOURCES);
    patch(id, { references });
    return references.slice(generator.references.length);
  }

  function removeReference(id: string, nodeId: string) {
    const generator = find(id);
    if (generator)
      patch(id, { references: generator.references.filter((reference) => reference !== nodeId) });
  }

  function discard(id: string) {
    all.value = all.value.filter((generator) => generator.id !== id);
    if (activeId.value === id) {
      activeId.value = undefined;
      picking.value = false;
    }
  }

  /** `sources` are the generator's references, pinned by the caller at the moment it was asked. */
  async function generate(id: string, sources: readonly CreationSource[]) {
    const generator = find(id);
    if (!generator || generator.submission || !generator.target || !generator.prompt.trim()) return;
    const kind = GENERATOR_KINDS[generator.kind];
    patch(id, { submission: "starting" });
    if (activeId.value === id) {
      activeId.value = undefined;
      picking.value = false;
    }
    const run = await startDerivationAction({
      actionId: kind.actionId,
      sources,
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

  watch(
    () => [space.activeSpaceId, space.nodes] as const,
    ([spaceId, nodes], [previousSpaceId, previous]) => {
      const ids = new Set(nodes.map(({ id }) => id));
      // Only a node that was here and is gone was deleted; a Space that is still loading lost nothing.
      const deleted =
        spaceId === previousSpaceId
          ? new Set(previous.filter(({ id }) => !ids.has(id)).map(({ id }) => id))
          : new Set();
      for (const { id, submission, references } of all.value) {
        // Done once the output it asked for stands in its frame.
        if (typeof submission === "object" && ids.has(runOutputNodeId(submission.runId))) discard(id);
        else if (references.some((reference) => deleted.has(reference)))
          patch(id, { references: references.filter((reference) => !deleted.has(reference)) });
      }
    },
  );
  watch(() => space.activeSpaceId, deactivate);

  return {
    generators,
    active,
    picking,
    open,
    takePromptFocus,
    activate,
    deactivate,
    release,
    startPicking,
    stopPicking,
    update,
    move,
    addReferences,
    removeReference,
    discard,
    generate,
  };
});
