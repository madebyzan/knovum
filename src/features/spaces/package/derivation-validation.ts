import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackageDerivations(manifest: SpacePackageManifest) {
  const nodeIds = new Set(manifest.nodes.map((node) => node.id));
  const outputs = new Set<string>();
  for (const derivation of manifest.derivations) {
    validateDerivationInputs(derivation, nodeIds);
    if (outputs.has(derivation.outputNodeId)) {
      throw new Error("Space package contains multiple derivations for one output");
    }
    outputs.add(derivation.outputNodeId);
  }
}

function validateDerivationInputs(
  derivation: SpacePackageManifest["derivations"][number],
  nodeIds: ReadonlySet<string>,
) {
  if (!nodeIds.has(derivation.outputNodeId))
    throw new Error("Space package derivation references an unavailable node");
  const inputIds = new Set<string>();
  const linkedInputs = new Set<string>();
  for (const input of derivation.inputs) {
    validateInputNode(input.nodeId, derivation.outputNodeId, nodeIds);
    registerUnique(inputIds, `${input.role}:${input.sourceIdentity}`, "duplicate input");
    if (input.nodeId !== null)
      registerUnique(linkedInputs, `${input.role}:${input.nodeId}`, "duplicate node link");
  }
}

function validateInputNode(nodeId: string | null, outputNodeId: string, nodeIds: ReadonlySet<string>) {
  if (nodeId !== null && (!nodeIds.has(nodeId) || nodeId === outputNodeId))
    throw new Error("Space package derivation references an unavailable node");
}

function registerUnique(identities: Set<string>, identity: string, problem: string) {
  if (identities.has(identity)) throw new Error(`Space package derivation contains a ${problem}`);
  identities.add(identity);
}
