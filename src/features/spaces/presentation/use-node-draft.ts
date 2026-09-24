import { computed, onBeforeUnmount, shallowRef, watch, type Ref } from "vue";

import { useNodeOperationGate } from "../canvas/node-operation-gate";
import { useNodeOperationDrafts, type NodeDraftKind } from "../session/node-operation-drafts";

export function useNodeDraft(nodeId: Readonly<Ref<string>>, kind: NodeDraftKind) {
  const gate = useNodeOperationGate();
  const drafts = useNodeOperationDrafts();
  const lease = shallowRef<number>();
  const available = computed(() => gate.allows("edit"));
  const canContinue = computed(() => lease.value !== undefined || available.value);

  function begin() {
    if (lease.value) return true;
    if (!available.value) return false;
    lease.value = drafts.begin(nodeId.value, kind);
    return lease.value !== undefined;
  }

  function end() {
    if (lease.value !== undefined) drafts.end(lease.value);
    lease.value = undefined;
  }

  watch(nodeId, end);
  onBeforeUnmount(end);
  return { available, canContinue, begin, end };
}
