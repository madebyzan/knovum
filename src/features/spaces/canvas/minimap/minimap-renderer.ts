import type { CanvasPalette } from "../use-canvas-palette";
import type { MinimapNode, MinimapRelation, MinimapScene, MinimapSection } from "./minimap-scene";
import type { MinimapDimensions, MinimapTransform } from "./minimap-transform";

const MAX_RENDERED_RELATIONS = 400;
const MIN_NODE_RENDER_PX = 2.5;
const MIN_SECTION_RENDER_PX = 4;

export interface MinimapRenderTarget {
  readonly ctx: CanvasRenderingContext2D;
  readonly transform: MinimapTransform;
  readonly palette: CanvasPalette;
  readonly dimensions: MinimapDimensions;
}

/** Renders sections, relations, and nodes onto a HiDPI 2D canvas in map pixel space. */
export function renderMinimapCanvas(target: MinimapRenderTarget, scene: MinimapScene) {
  const { ctx, dimensions } = target;
  ctx.clearRect(0, 0, dimensions.width, dimensions.height);
  for (const section of scene.sections) {
    drawSection(target, section);
  }
  if (scene.relations.length <= MAX_RENDERED_RELATIONS) {
    drawRelations(target, scene.relations);
  }
  for (const node of scene.nodes) {
    drawNode(target, node, scene.hasFocus);
  }
}

function drawSection(target: MinimapRenderTarget, section: MinimapSection) {
  const { ctx, transform, palette } = target;
  const x = section.bounds.left * transform.scale + transform.offsetX;
  const y = section.bounds.top * transform.scale + transform.offsetY;
  const width = Math.max(
    MIN_SECTION_RENDER_PX,
    (section.bounds.right - section.bounds.left) * transform.scale,
  );
  const height = Math.max(
    MIN_SECTION_RENDER_PX,
    (section.bounds.bottom - section.bounds.top) * transform.scale,
  );
  const fill =
    section.customFill ??
    (section.fillPreset ? palette.contentFillByPreset[section.fillPreset] : palette.panel);
  ctx.save();
  ctx.beginPath();
  ctx.roundRect(x, y, width, height, Math.min(4, width / 4, height / 4));
  ctx.globalAlpha = section.collapsed ? 0.42 : 0.24;
  ctx.fillStyle = fill;
  ctx.fill();
  ctx.globalAlpha = section.selected ? 0.95 : 0.7;
  ctx.strokeStyle = section.selected ? palette.accent : palette.borderStrong;
  ctx.lineWidth = section.selected ? 1.25 : 0.75;
  ctx.stroke();
  ctx.restore();
}

function drawRelations(target: MinimapRenderTarget, relations: readonly MinimapRelation[]) {
  const { ctx, transform, palette } = target;
  ctx.save();
  ctx.lineWidth = 0.85;
  ctx.globalAlpha = 0.34;
  for (const relation of relations) {
    ctx.strokeStyle = palette.relationByColor[relation.color];
    ctx.beginPath();
    ctx.moveTo(
      relation.from.x * transform.scale + transform.offsetX,
      relation.from.y * transform.scale + transform.offsetY,
    );
    ctx.lineTo(
      relation.to.x * transform.scale + transform.offsetX,
      relation.to.y * transform.scale + transform.offsetY,
    );
    ctx.stroke();
  }
  ctx.restore();
}

function drawNode(target: MinimapRenderTarget, node: MinimapNode, hasFocus: boolean) {
  const { ctx, transform } = target;
  const x = node.x * transform.scale + transform.offsetX;
  const y = node.y * transform.scale + transform.offsetY;
  const width = Math.max(MIN_NODE_RENDER_PX, node.width * transform.scale);
  const height = Math.max(node.kind === "text" ? 1.75 : MIN_NODE_RENDER_PX, node.height * transform.scale);
  ctx.save();
  if (node.rotation !== 0) {
    ctx.translate(x, y);
    ctx.rotate((node.rotation * Math.PI) / 180);
    drawNodeShape(target, node, { x: 0, y: 0, width, height }, hasFocus);
  } else {
    drawNodeShape(target, node, { x, y, width, height }, hasFocus);
  }
  ctx.restore();
}

interface NodeRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

function drawNodeShape(target: MinimapRenderTarget, node: MinimapNode, rect: NodeRect, hasFocus: boolean) {
  const { ctx, palette } = target;
  const highlighted = node.selected || node.focused;
  const dimmed = hasFocus && !highlighted;
  ctx.globalAlpha = dimmed ? 0.35 : 0.92;
  if (node.kind === "text") {
    drawTextShape(ctx, rect, highlighted ? palette.accent : palette.foregroundSecondary);
    return;
  }
  ctx.beginPath();
  ctx.roundRect(rect.x, rect.y, rect.width, rect.height, Math.min(2.5, rect.width / 4, rect.height / 4));
  ctx.fillStyle = resolveNodeFill(node, palette);
  ctx.fill();
  ctx.globalAlpha = highlighted ? 1 : dimmed ? 0.35 : 0.78;
  ctx.strokeStyle = highlighted ? palette.accent : palette.borderStrong;
  ctx.lineWidth = highlighted ? 1.25 : 0.75;
  ctx.stroke();
}

function drawTextShape(ctx: CanvasRenderingContext2D, rect: NodeRect, fillStyle: string) {
  ctx.fillStyle = fillStyle;
  if (rect.height < 6) {
    ctx.fillRect(rect.x, rect.y, rect.width, Math.min(rect.height, 2));
    return;
  }
  const lineHeight = Math.min(2.2, rect.height * 0.28);
  ctx.fillRect(rect.x, rect.y, rect.width, lineHeight);
  ctx.fillRect(rect.x, rect.y + lineHeight + 1.5, rect.width * 0.68, lineHeight);
}

function resolveNodeFill(node: MinimapNode, palette: CanvasPalette): string {
  if (node.customFill) return node.customFill;
  if (node.fillPreset) return palette.contentFillByPreset[node.fillPreset];
  if (node.kind === "note") return palette.contentFillByPreset.yellow;
  if (node.kind === "image" || node.kind === "video") return palette.contentFillByPreset.gray;
  return palette.panel;
}
