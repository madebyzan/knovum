import type { useSpaceStore } from "../../session/space-store";
import type { GroundedCitation } from "../model/grounded-citation";
import { isLocatorCurrent } from "../model/location";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export function groundedCitationCurrent(store: SpaceStore, citation: GroundedCitation) {
  if (citation.state !== "current") return false;
  const node = store.nodes.find((candidate) => candidate.id === citation.nodeId);
  return isLocatorCurrent(citation.locator, node?.data);
}
