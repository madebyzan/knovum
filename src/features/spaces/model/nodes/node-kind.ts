import type { SpaceNodeKind } from "./node-payload";
import { spaceNodePayloadSchema } from "./schema/node-variants";

export const SPACE_NODE_KINDS = spaceNodePayloadSchema.options.map((option) => option.shape.kind.value) as [
  SpaceNodeKind,
  ...SpaceNodeKind[],
];
