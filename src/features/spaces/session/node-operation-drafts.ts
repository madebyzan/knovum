import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

export type NodeDraftKind =
  | "title"
  | "portal"
  | "outline-title"
  | "knowledge-object"
  | "media-summary"
  | "media-chapter"
  | "transcript"
  | "credential";

interface NodeDraftLease {
  id: number;
  nodeId: string;
  kind: NodeDraftKind;
}

export const useNodeOperationDrafts = defineStore("spaces-node-operation-drafts", () => {
  const active = shallowRef<NodeDraftLease>();
  let nextId = 0;

  function begin(nodeId: string, kind: NodeDraftKind) {
    if (active.value) return undefined;
    const id = ++nextId;
    active.value = { id, nodeId, kind };
    return id;
  }

  function end(id: number) {
    if (active.value?.id === id) active.value = undefined;
  }

  return { active: readonly(active), begin, end };
});
