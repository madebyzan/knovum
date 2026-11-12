import { computed, reactive, toRef, watch, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { speakerReferences, type SpeakerReference } from "../speaker-references";
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
        get maxDurationMs() {
          return form.referenceDuration?.maxMs;
        },
      },
      toRef(form, "reference"),
    ),
  );
  watch(
    () => reference.jobId,
    (job) => (form.preparingReference = Boolean(job)),
  );
  /** Each speaker the node's current transcript tells apart, as a selection to clone them from. */
  const speakers = computed(() => {
    const data = node.value.data;
    if (data.kind !== "audio" && data.kind !== "video") return [];
    const knowledge = data.mediaKnowledge;
    if (!knowledge?.transcript || knowledge.sourceRevision !== data.sourceRevision) return [];
    return speakerReferences(knowledge.transcript, reference.maxSeconds * 1000);
  });
  function selectSpeaker(speaker: SpeakerReference) {
    reference.start = String(speaker.startMs / 1000);
    reference.end = String(speaker.endMs / 1000);
  }
  async function save(candidateId: string, name: string) {
    await form.save(candidateId, name);
  }
  return { form, reference, speakers, selectSpeaker, save };
}
