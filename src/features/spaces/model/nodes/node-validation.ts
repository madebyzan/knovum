import type { SpaceNodePayload } from "./node-payload";
import { spaceNodePayloadSchema } from "./schema/node-variants";

export function validateSpaceNodePayload(value: unknown): SpaceNodePayload {
  return spaceNodePayloadSchema.parse(value);
}
