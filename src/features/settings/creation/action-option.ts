import type { RoutingPurpose } from "../../../shared/contracts/generated/RoutingPurpose";

export interface GenerationActionOption {
  id: string;
  routingPurpose: RoutingPurpose | null;
}
