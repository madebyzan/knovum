import type { SpaceNodePayload } from "../node-payload";
import { spaceNodePayloadSchema } from "./node-variants";

export function validateSpaceNodePayload(value: unknown): SpaceNodePayload {
  return spaceNodePayloadSchema.parse(value);
}
