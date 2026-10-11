import { reactive, toRef, watch, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useVoiceCandidateForm } from "../use-voice-candidate-form";
import { useVoiceReferenceEditor } from "../use-voice-reference-editor";

export function useCanvasVoiceClone(node: Readonly<Ref<SpaceCanvasNode>>) {
  const form = reactive(useVoiceCandidateForm(() => "reference"));
  const reference = reactive(
    useVoiceReferenceEditor(
      {
        get sourceNode() {
          return node.value;
        },
        get language() {
          return form.language;
        },
        get disabled() {
          return form.busy;
        },
      },
      toRef(form, "reference"),
    ),
  );
  watch(
    () => reference.jobId,
    (job) => (form.preparingReference = Boolean(job)),
  );
  async function save(candidateId: string, name: string) {
    await form.save(candidateId, name);
  }
  return { form, reference, save };
}
