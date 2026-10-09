import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

interface NodeDraftLease {
  id: number;
  nodeId: string;
}

export const useNodeOperationDrafts = defineStore("spaces-node-operation-drafts", () => {
  const active = shallowRef<NodeDraftLease>();
  let nextId = 0;

  function begin(nodeId: string) {
    if (active.value) return undefined;
    const id = ++nextId;
    active.value = { id, nodeId };
    return id;
  }

  function end(id: number) {
    if (active.value?.id === id) active.value = undefined;
  }

  return { active: readonly(active), begin, end };
});
