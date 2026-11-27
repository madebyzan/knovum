import type { Editor } from "@tiptap/core";

export interface TopLevelBlockMetric {
  readonly index: number;
  readonly pos: number;
  readonly top: number;
  readonly bottom: number;
  readonly handleTop: number;
  readonly label: string;
}

export interface BlockDropSlot {
  readonly targetIndex: number;
  readonly lineTop: number;
}

const HANDLE_HEIGHT = 22;
const BOX_BLOCK_TYPES = new Set(["codeBlock", "callout", "table"]);
const LIST_BLOCK_TYPES = new Set(["bulletList", "orderedList", "taskList"]);

function computeOpticalHandleTop(dom: HTMLElement, nodeType: string, top: number, height: number): number {
  if (nodeType === "horizontalRule") return Math.round(top + (height - HANDLE_HEIGHT) / 2);
  if (BOX_BLOCK_TYPES.has(nodeType)) return Math.round(top + 8);
  const sample = LIST_BLOCK_TYPES.has(nodeType)
    ? (dom.querySelector<HTMLElement>("li > p, li > div, li") ?? dom)
    : dom;
  const style = getComputedStyle(sample);
  const fontSize = Number.parseFloat(style.fontSize) || 15;
  const parsedLineHeight = Number.parseFloat(style.lineHeight);
  const lineHeight = Number.isFinite(parsedLineHeight) ? parsedLineHeight : fontSize * 1.5;
  const paddingTop = Number.parseFloat(style.paddingTop) || 0;
  return Math.round(top + paddingTop + Math.max(0, (lineHeight - HANDLE_HEIGHT) / 2));
}

export function measureTopLevelBlocks(editor: Editor, hostEl: HTMLElement): readonly TopLevelBlockMetric[] {
  if (editor.isDestroyed) return [];
  const hostTop = hostEl.getBoundingClientRect().top;
  const metrics: TopLevelBlockMetric[] = [];
  editor.state.doc.forEach((node, offset, index) => {
    const dom = editor.view.nodeDOM(offset);
    if (!(dom instanceof HTMLElement)) return;
    const rect = dom.getBoundingClientRect();
    const top = rect.top - hostTop;
    const bottom = rect.bottom - hostTop;
    const text = node.textContent.trim().replace(/\s+/gu, " ");
    metrics.push({
      index,
      pos: offset,
      top,
      bottom,
      handleTop: computeOpticalHandleTop(dom, node.type.name, top, rect.height),
      label: text.slice(0, 48),
    });
  });
  return metrics;
}

export function hitTestHoveredBlock(
  metrics: readonly TopLevelBlockMetric[],
  yInHost: number,
): TopLevelBlockMetric | null {
  let nearest: TopLevelBlockMetric | null = null;
  let nearestDistance = 16;
  for (const item of metrics) {
    if (yInHost >= item.top - 4 && yInHost <= item.bottom + 4) return item;
    const dist = Math.min(Math.abs(yInHost - item.top), Math.abs(yInHost - item.bottom));
    if (dist < nearestDistance) {
      nearestDistance = dist;
      nearest = item;
    }
  }
  return nearest;
}

export function resolveBlockDropSlot(
  metrics: readonly TopLevelBlockMetric[],
  yInHost: number,
): BlockDropSlot | null {
  if (!metrics.length) return null;
  for (let index = 0; index < metrics.length; index += 1) {
    const current = metrics[index]!;
    const midpoint = (current.top + current.bottom) / 2;
    if (yInHost <= midpoint) {
      const previous = metrics[index - 1];
      const lineTop = previous ? (previous.bottom + current.top) / 2 : current.top - 4;
      return { targetIndex: index, lineTop: Math.round(lineTop) };
    }
  }
  const last = metrics[metrics.length - 1]!;
  return { targetIndex: metrics.length, lineTop: Math.round(last.bottom + 4) };
}
