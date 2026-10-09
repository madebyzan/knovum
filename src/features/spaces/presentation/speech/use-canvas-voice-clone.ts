import { reactive, toRef, watch, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useVoiceCandidateForm } from "../../speech/use-voice-candidate-form";
import { useVoiceReferenceEditor } from "../../speech/use-voice-reference-editor";

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
  function setRange(start: number, end: number) {
    reference.start = String(start);
    reference.end = String(end);
  }
  async function save(candidateId: string, name: string) {
    await form.save(candidateId, name, form.authorized);
  }
  return { form, reference, setRange, save };
}
