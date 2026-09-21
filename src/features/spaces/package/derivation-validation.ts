import type { SpacePackageManifest } from "./manifest-schema";

type PackageDerivation = SpacePackageManifest["derivations"][number];

export function validatePackageDerivations(manifest: SpacePackageManifest) {
  const nodeIds = new Set(manifest.nodes.map((node) => node.id));
  const outputs = new Set<string>();
  for (const derivation of manifest.derivations) {
    requireAvailableOutput(derivation, nodeIds);
    validateDerivationInputs(derivation, nodeIds);
    if (outputs.has(derivation.outputNodeId)) {
      throw new Error("Space package contains multiple derivations for one output");
    }
    outputs.add(derivation.outputNodeId);
  }
}

function requireAvailableOutput(derivation: PackageDerivation, nodeIds: ReadonlySet<string>) {
  if (!nodeIds.has(derivation.outputNodeId)) {
    throw new Error("Space package derivation references an unavailable node");
  }
}

function validateDerivationInputs(derivation: PackageDerivation, nodeIds: ReadonlySet<string>) {
  const inputIds = new Set<string>();
  const linkedInputs = new Set<string>();
  for (const input of derivation.inputs) {
    requireAvailableInput(input.nodeId, derivation.outputNodeId, nodeIds);
    const identity = `${input.role}:${input.sourceIdentity}`;
    if (inputIds.has(identity)) throw new Error("Space package derivation contains a duplicate input");
    inputIds.add(identity);
    if (input.nodeId === null) continue;
    const link = `${input.role}:${input.nodeId}`;
    if (linkedInputs.has(link)) {
      throw new Error("Space package derivation contains a duplicate node link");
    }
    linkedInputs.add(link);
  }
}

function requireAvailableInput(
  nodeId: string | null,
  outputNodeId: string,
  nodeIds: ReadonlySet<string>,
) {
  if (nodeId !== null && (!nodeIds.has(nodeId) || nodeId === outputNodeId)) {
    throw new Error("Space package derivation references an unavailable node");
  }
}
