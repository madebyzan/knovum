import { shallowRef } from "vue";

import { updateTextOverlay } from "../edits/text";
import { setClipTransform } from "../edits/trim";
import { layerGeometry, scaleFromSize, type LayerGeometry } from "../geometry";
import type { VideoClip, VideoEditProject, VideoTextOverlay } from "../project";
import type { VideoSelectionController } from "./use-video-selection";

export type StageHandle = "move" | "scale" | "rotate";

interface StageGestureOptions {
  project: () => VideoEditProject | undefined;
  selection: VideoSelectionController;
  /** CSS pixels per output pixel on the stage. */
  scale: () => number;
  commit: (project: VideoEditProject, mergeKey?: string) => void;
}

interface Session {
  handle: StageHandle;
  originX: number;
  originY: number;
  clip: VideoClip | undefined;
  textId: string | undefined;
  startX: number;
  startY: number;
  startRotation: number;
  startWidth: number;
  pointerAngle: number;
  pointerDistance: number;
}

const ROTATION_SNAP_DEGREES = 3;

/** Direct manipulation on the stage: drag to position, corner to resize, handle to rotate. */
export function useVideoStageGestures(options: StageGestureOptions) {
  const session = shallowRef<Session>();

  function begin(event: PointerEvent, handle: StageHandle) {
    const project = options.project();
    const clip = options.selection.clip.value;
    const text = options.selection.text.value;
    if (event.button !== 0 || !project || !(clip ?? text)) return;
    event.stopPropagation();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    session.value = startSession({ handle, event, project, clip, text, scale: options.scale() });
  }

  function move(event: PointerEvent) {
    const current = session.value;
    const project = options.project();
    if (!current || !project) return;
    const next =
      current.handle === "move" ? translate(project, current, event) : reshape(project, current, event);
    if (next) options.commit(next, `stage:${current.clip?.id ?? current.textId}`);
  }

  function translate(project: VideoEditProject, current: Session, event: PointerEvent) {
    const deltaX = (event.clientX - current.originX) / options.scale() / project.output.width;
    const deltaY = (event.clientY - current.originY) / options.scale() / project.output.height;
    if (current.clip)
      return setClipTransform(project, current.clip.id, {
        x: snapTo(current.startX + deltaX, 0, event.altKey),
        y: snapTo(current.startY + deltaY, 0, event.altKey),
      });
    if (!current.textId) return undefined;
    return updateTextOverlay(project, current.textId, {
      x: clampUnit(snapTo(current.startX + deltaX, 0.5, event.altKey)),
      y: clampUnit(snapTo(current.startY + deltaY, 0.5, event.altKey)),
    });
  }

  function reshape(project: VideoEditProject, current: Session, event: PointerEvent) {
    const clip = current.clip;
    const asset = clip && assetOf(project, clip);
    if (!clip || !asset) return undefined;
    const geometry = layerGeometry(asset, clip.transform, project.output);
    if (current.handle === "rotate") {
      const delta = angleOf(event, geometry, options.scale()) - current.pointerAngle;
      return setClipTransform(project, clip.id, {
        rotation: snapRotation(current.startRotation + delta, event.altKey),
      });
    }
    const ratio = distanceOf(event, geometry, options.scale()) / Math.max(1, current.pointerDistance);
    const width = Math.max(8, current.startWidth * ratio);
    return setClipTransform(project, clip.id, {
      scale: Math.min(8, Math.max(0.05, scaleFromSize(asset, project.output, clip.transform.fit, width))),
    });
  }

  function finish() {
    session.value = undefined;
  }

  return { session, begin, move, finish, cancel: finish };
}

interface SessionInput {
  handle: StageHandle;
  event: PointerEvent;
  project: VideoEditProject;
  clip: VideoClip | undefined;
  text: VideoTextOverlay | undefined;
  scale: number;
}

/** Captures everything the drag needs up front, so every move is a pure delta. */
function startSession(input: SessionInput): Session {
  const { clip, text, event } = input;
  const geometry = clip
    ? layerGeometry(assetOf(input.project, clip)!, clip.transform, input.project.output)
    : undefined;
  return {
    handle: input.handle,
    originX: event.clientX,
    originY: event.clientY,
    clip,
    textId: text?.id,
    ...anchorOf(clip, text),
    ...pointerOf(event, geometry, input.scale),
  };
}

function anchorOf(clip: VideoClip | undefined, text: VideoTextOverlay | undefined) {
  if (clip)
    return {
      startX: clip.transform.x,
      startY: clip.transform.y,
      startRotation: clip.transform.rotation,
    };
  return { startX: text?.x ?? 0.5, startY: text?.y ?? 0.5, startRotation: 0 };
}

function pointerOf(event: PointerEvent, geometry: LayerGeometry | undefined, scale: number) {
  if (!geometry) return { startWidth: 1, pointerAngle: 0, pointerDistance: 1 };
  return {
    startWidth: geometry.width,
    pointerAngle: angleOf(event, geometry, scale),
    pointerDistance: distanceOf(event, geometry, scale),
  };
}

function snapTo(value: number, anchor: number, disabled: boolean) {
  return !disabled && Math.abs(value - anchor) < 0.01 ? anchor : Number(value.toFixed(4));
}

function clampUnit(value: number) {
  return Math.min(1.5, Math.max(-0.5, value));
}

function assetOf(project: VideoEditProject, clip: VideoClip) {
  return project.assets.find(({ nodeId }) => nodeId === clip.assetId);
}

function snapRotation(value: number, disabled: boolean) {
  const wrapped = ((value + 180) % 360) - 180;
  if (disabled) return Number(wrapped.toFixed(2));
  const quarter = Math.round(wrapped / 90) * 90;
  return Math.abs(wrapped - quarter) <= ROTATION_SNAP_DEGREES
    ? Math.max(-180, Math.min(180, quarter))
    : Number(wrapped.toFixed(2));
}

function stageOrigin(event: PointerEvent, geometry: LayerGeometry, scale: number) {
  const frame = (event.currentTarget as HTMLElement).closest<HTMLElement>("[data-video-stage-frame]");
  const rect = frame?.getBoundingClientRect();
  if (!rect) return undefined;
  return {
    x: event.clientX - rect.left - geometry.centerX * scale,
    y: event.clientY - rect.top - geometry.centerY * scale,
  };
}

function angleOf(event: PointerEvent, geometry: LayerGeometry, scale: number) {
  const origin = stageOrigin(event, geometry, scale);
  return origin ? (Math.atan2(origin.y, origin.x) * 180) / Math.PI : 0;
}

function distanceOf(event: PointerEvent, geometry: LayerGeometry, scale: number) {
  const origin = stageOrigin(event, geometry, scale);
  return origin ? Math.max(1, Math.hypot(origin.x, origin.y)) : 1;
}
