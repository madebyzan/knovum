import pLimit from "p-limit";

import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImportSourceAccess } from "./asset-operations";
import { layoutImportGrid } from "./import-placement";
import {
  commitPreparedSpaceImport,
  prepareSpaceImportSource,
  type PreparedSpaceImportOperation,
  type SpaceImportContext,
  type SpaceImportSessionLabels,
} from "./import-session";
import { importedSpaceNodePayload } from "./imported-asset-payload";

const importConcurrency = pLimit(3);

export interface SpaceImportBatchRequest<Key> {
  key: Key;
  path: string;
  sourceAccess: SpaceImportSourceAccess;
  position?: { x: number; y: number };
}

export type SpaceImportBatchOutcome<Key> =
  | { status: "imported"; key: Key; nodes: [SpaceCanvasNode, ...SpaceCanvasNode[]] }
  | { status: "cancelled"; key: Key }
  | { status: "failed"; key: Key; error: unknown };

export async function importSpaceBatch<Key>(input: {
  context: SpaceImportContext;
  spaceId: string;
  requests: readonly SpaceImportBatchRequest<Key>[];
  origin: { x: number; y: number };
  labels: SpaceImportSessionLabels;
}): Promise<SpaceImportBatchOutcome<Key>[]> {
  const { context, labels, origin, requests, spaceId } = input;
  await context.ensureImportTarget(spaceId);
  const prepared = await Promise.all(
    requests.map((request) => importConcurrency(() => prepareBatchItem(request, labels))),
  );
  const positions = batchPositions(prepared, origin);
  return Promise.all(
    prepared.map((item, index) =>
      item.status === "failed"
        ? item
        : importConcurrency(() =>
            commitBatchItem({
              context,
              operation: item.operation,
              position: batchItemPosition(item, index, positions),
              request: item.request,
              spaceId,
            }),
          ),
    ),
  );
}

function batchPositions<Key>(items: readonly PreparedBatchItem<Key>[], origin: { x: number; y: number }) {
  const automatic = items.flatMap((item, index) =>
    item.status === "prepared" && item.request.position === undefined ? [{ index, item }] : [],
  );
  const positions = layoutImportGrid(
    automatic.map(({ item }) => spaceNodeDimensions(importedSpaceNodePayload(item.operation.prepared.asset))),
    origin,
  );
  return new Map(automatic.map(({ index }, positionIndex) => [index, positions[positionIndex]!]));
}

function batchItemPosition<Key>(
  item: PreparedBatchSuccess<Key>,
  index: number,
  positions: ReadonlyMap<number, { x: number; y: number }>,
) {
  if (item.request.position) return item.request.position;
  const position = positions.get(index);
  if (!position) throw new Error("Prepared import has no automatic canvas position");
  return position;
}

async function prepareBatchItem<Key>(
  request: SpaceImportBatchRequest<Key>,
  labels: SpaceImportSessionLabels,
): Promise<PreparedBatchItem<Key>> {
  try {
    return {
      status: "prepared",
      request,
      operation: await prepareSpaceImportSource(request.path, labels, request.sourceAccess),
    };
  } catch (error) {
    return { status: "failed", key: request.key, error };
  }
}

async function commitBatchItem<Key>(input: {
  context: SpaceImportContext;
  spaceId: string;
  request: SpaceImportBatchRequest<Key>;
  operation: PreparedSpaceImportOperation;
  position: { x: number; y: number };
}): Promise<SpaceImportBatchOutcome<Key>> {
  const { context, operation, position, request, spaceId } = input;
  try {
    const nodes = await commitPreparedSpaceImport({
      context,
      spaceId,
      operation,
      position,
      reveal: false,
    });
    const first = nodes[0];
    return first
      ? { status: "imported", key: request.key, nodes: [first, ...nodes.slice(1)] }
      : { status: "cancelled", key: request.key };
  } catch (error) {
    return { status: "failed", key: request.key, error };
  }
}

interface PreparedBatchSuccess<Key> {
  status: "prepared";
  request: SpaceImportBatchRequest<Key>;
  operation: PreparedSpaceImportOperation;
}

type PreparedBatchItem<Key> =
  | PreparedBatchSuccess<Key>
  | Extract<SpaceImportBatchOutcome<Key>, { status: "failed" }>;
