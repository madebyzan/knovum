import Konva from "konva";

import { updateTextOverlay } from "../edits/text";
import { setClipTransform } from "../edits/trim";
import { scaleFromSize } from "../geometry";
import type { VideoEditProject } from "../project";
import type { StageScene } from "./konva-scene";
import { readStagePalette } from "./stage-palette";

export interface StageTarget {
  kind: "clip" | "text";
  id: string;
}

interface StageEditingOptions {
  project: () => VideoEditProject | undefined;
  commit: (project: VideoEditProject, mergeKey?: string) => void;
  onSelect: (target: StageTarget | undefined) => void;
}

const ROTATION_SNAPS = [0, 90, 180, 270];
const SNAP_TOLERANCE = 3;
const MIN_WIDTH = 8;
const SCALE_RANGE = { min: 0.05, max: 8 };
/** Handle metrics in screen pixels; they are divided by the stage scale to stay that size. */
const HANDLE = { anchor: 9, stroke: 1, padding: 2, rotateOffset: 26 };

function targetOf(node: Konva.Node): StageTarget | undefined {
  const id = node.id();
  if (id.startsWith("picture:")) return { kind: "clip", id: id.slice("picture:".length) };
  if (id.startsWith("text:")) return { kind: "text", id: id.slice("text:".length) };
  return undefined;
}

function fraction(value: number) {
  return Number(value.toFixed(4));
}

function moveClip(project: VideoEditProject, id: string, node: Konva.Node) {
  return setClipTransform(project, id, {
    x: fraction((node.x() - project.output.width / 2) / project.output.width),
    y: fraction((node.y() - project.output.height / 2) / project.output.height),
  });
}

function place(value: number) {
  return Math.min(1.5, Math.max(-0.5, fraction(value)));
}

function moveText(project: VideoEditProject, id: string, node: Konva.Node) {
  return updateTextOverlay(project, id, {
    x: place(node.x() / project.output.width),
    y: place(node.y() / project.output.height),
  });
}

function reshapeClip(project: VideoEditProject, id: string, node: Konva.Node) {
  const clip = project.videoTrack.clips.find((candidate) => candidate.id === id);
  const asset = clip && project.assets.find(({ nodeId }) => nodeId === clip.assetId);
  if (!clip || !asset) return undefined;
  const width = Math.max(MIN_WIDTH, node.width() * Math.abs(node.scaleX()));
  const scale = scaleFromSize(asset, project.output, clip.transform.fit, width);
  return setClipTransform(project, id, {
    rotation: Number(node.rotation().toFixed(2)),
    scale: Math.min(SCALE_RANGE.max, Math.max(SCALE_RANGE.min, scale)),
  });
}

function edited(project: VideoEditProject, target: StageTarget, node: Konva.Node, reshaping: boolean) {
  if (target.kind === "text") return reshaping ? undefined : moveText(project, target.id, node);
  return reshaping ? reshapeClip(project, target.id, node) : moveClip(project, target.id, node);
}

/**
 * Direct manipulation on the stage. Konva's transformer owns the handles, hit areas, rotation
 * snapping and pointer maths; this module only translates what the user did back into the project,
 * which stays the single source of truth for every transform.
 */
export function createStageEditing(options: StageEditingOptions) {
  const palette = readStagePalette();
  const transformer = new Konva.Transformer({
    keepRatio: true,
    centeredScaling: true,
    rotationSnaps: ROTATION_SNAPS,
    rotationSnapTolerance: SNAP_TOLERANCE,
    enabledAnchors: ["top-left", "top-right", "bottom-left", "bottom-right"],
    borderStroke: palette.accent,
    anchorStroke: palette.accent,
    anchorFill: palette.anchor,
    ignoreStroke: true,
    flipEnabled: false,
  });
  let scene: StageScene | undefined;

  function apply(node: Konva.Node, reshaping: boolean) {
    const project = options.project();
    const target = targetOf(node);
    if (!project || !target) return;
    const next = edited(project, target, node, reshaping);
    if (next) options.commit(next, `stage:${target.id}`);
  }

  return {
    attach(next: StageScene) {
      scene = next;
      next.overlay.add(transformer);
      next.stage.on("click tap", (event) => options.onSelect(targetOf(event.target)));
      next.stage.on("dragmove", (event) => apply(event.target, false));
      transformer.on("transform", () => {
        const node = transformer.nodes()[0];
        if (node) apply(node, true);
      });
    },
    /** Points the handles at the selected node, or hides them when nothing is selected. */
    highlight(target: StageTarget | undefined) {
      const prefix = target?.kind === "clip" ? "picture:" : "text:";
      const node = target ? scene?.stage.findOne(`#${prefix}${target.id}`) : undefined;
      transformer.nodes(node ? [node] : []);
      transformer.resizeEnabled(target?.kind === "clip");
      transformer.rotateEnabled(target?.kind === "clip");
      scene?.overlay.batchDraw();
    },
    /** Keeps the handles the same size on screen at any stage zoom. */
    rescale(scale: number) {
      const ratio = 1 / Math.max(0.01, scale);
      transformer.setAttrs({
        anchorSize: HANDLE.anchor * ratio,
        anchorStrokeWidth: HANDLE.stroke * ratio,
        anchorCornerRadius: (HANDLE.anchor / 2) * ratio,
        borderStrokeWidth: HANDLE.stroke * ratio,
        padding: HANDLE.padding * ratio,
        rotateAnchorOffset: HANDLE.rotateOffset * ratio,
      });
    },
    dispose() {
      transformer.destroy();
      scene = undefined;
    },
  };
}
