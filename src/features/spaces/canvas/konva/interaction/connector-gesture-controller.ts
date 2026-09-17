import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasPoint } from "../../contract/canvas-editor";
import {
  connectorAnchor,
  frameContainsPoint,
  nearestConnectorTarget,
  type CanvasConnectorSide,
} from "../scene/connector-geometry";

export type { CanvasConnectorSide } from "../scene/connector-geometry";

export interface CanvasConnectorDraft {
  sourceNodeId: string;
  sourceSide: CanvasConnectorSide;
  from: CanvasPoint;
  to: CanvasPoint;
  target?: CanvasConnectorTarget;
}

export interface CanvasConnectorTarget {
  nodeId: string;
  side: CanvasConnectorSide;
  anchor: CanvasPoint;
}

export interface CanvasConnectorCommit {
  sourceNodeId: string;
  sourceSide: CanvasConnectorSide;
  sourceEnd: "none";
  targetNodeId: string;
  targetSide: CanvasConnectorSide;
  targetEnd: "arrow";
}

export class ConnectorGestureController {
  private current: CanvasConnectorDraft | undefined;

  constructor(
    private readonly layouts: () => readonly CanvasNodeLayoutV1[],
    private readonly changed: (draft?: CanvasConnectorDraft) => void,
    private readonly zoom: () => number = () => 1,
  ) {}

  begin(nodeId: string, side: CanvasConnectorSide) {
    const layout = this.layouts().find((item) => item.nodeId === nodeId);
    if (!layout) return;
    const from = connectorAnchor(layout.frame, side);
    this.current = { sourceNodeId: nodeId, sourceSide: side, from, to: from };
    this.changed(this.current);
  }

  move(point: CanvasPoint) {
    if (!this.current) return false;
    const target = this.resolveTarget(point);
    this.current = {
      sourceNodeId: this.current.sourceNodeId,
      sourceSide: this.current.sourceSide,
      from: this.current.from,
      to: target?.anchor ?? point,
      ...(target ? { target } : {}),
    };
    this.changed(this.current);
    return true;
  }

  finish(point: CanvasPoint): CanvasConnectorCommit | undefined {
    const draft = this.current;
    const target = draft?.target ?? this.resolveTarget(point);
    this.current = undefined;
    this.changed();
    if (!draft) return undefined;
    if (!target) return undefined;
    return {
      sourceNodeId: draft.sourceNodeId,
      sourceSide: draft.sourceSide,
      sourceEnd: "none" as const,
      targetNodeId: target.nodeId,
      targetSide: target.side,
      targetEnd: "arrow" as const,
    };
  }

  cancel() {
    if (!this.current) return;
    this.current = undefined;
    this.changed();
  }

  get active() {
    return this.current !== undefined;
  }

  private resolveTarget(point: CanvasPoint): CanvasConnectorTarget | undefined {
    if (!this.current) return undefined;
    const candidates = this.layouts().filter(({ nodeId }) => nodeId !== this.current?.sourceNodeId);
    const containing = candidates.toReversed().find(({ frame }) => frameContainsPoint(frame, point));
    return nearestConnectorTarget(
      containing ? [containing] : candidates,
      point,
      containing ? Number.POSITIVE_INFINITY : 18 / this.zoom(),
    );
  }
}
