import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackageDerivations(manifest: SpacePackageManifest) {
  const nodeIds = new Set(manifest.nodes.map((node) => node.id));
  const outputs = new Set<string>();
  for (const derivation of manifest.derivations) {
    if (!nodeIds.has(derivation.outputNodeId)) {
      throw new Error("Space package derivation references an unavailable node");
    }
    const inputIds = new Set<string>();
    const linkedInputs = new Set<string>();
    for (const input of derivation.inputs) {
      if (input.nodeId !== null && (!nodeIds.has(input.nodeId) || input.nodeId === derivation.outputNodeId)) {
        throw new Error("Space package derivation references an unavailable node");
      }
      const identity = `${input.role}:${input.sourceIdentity}`;
      if (inputIds.has(identity)) throw new Error("Space package derivation contains a duplicate input");
      inputIds.add(identity);
      if (input.nodeId !== null) {
        const link = `${input.role}:${input.nodeId}`;
        if (linkedInputs.has(link))
          throw new Error("Space package derivation contains a duplicate node link");
        linkedInputs.add(link);
      }
    }
    if (outputs.has(derivation.outputNodeId)) {
      throw new Error("Space package contains multiple derivations for one output");
    }
    outputs.add(derivation.outputNodeId);
  }
}
