import type { SpaceNodeKind } from "./node-payload";
import { spaceNodePayloadDiscriminatedSchema } from "./schema/node-variants";

export const SPACE_NODE_KINDS = spaceNodePayloadDiscriminatedSchema.options.map(
  (option) => option.shape.kind.value,
) as [SpaceNodeKind, ...SpaceNodeKind[]];
