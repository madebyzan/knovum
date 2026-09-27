import type { SpaceNodePayload } from "./node-payload";

export interface NodePayloadChange {
  id: string;
  before: SpaceNodePayload;
  after: SpaceNodePayload;
}
