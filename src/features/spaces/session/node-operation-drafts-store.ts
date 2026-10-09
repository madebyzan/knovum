import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

interface NodeDraftLease {
  id: number;
  nodeId: string;
}

export const useNodeOperationDraftsStore = defineStore("spaces-node-operation-drafts", () => {
  const active = shallowRef<NodeDraftLease>();
  let nextLeaseId = 0;

  function begin(nodeId: string) {
    if (active.value) return undefined;
    const leaseId = ++nextLeaseId;
    active.value = { id: leaseId, nodeId };
    return leaseId;
  }

  function end(leaseId: number) {
    if (active.value?.id === leaseId) active.value = undefined;
  }

  return { active: readonly(active), begin, end };
});
