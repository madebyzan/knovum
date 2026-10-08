import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { LocalModelReferenceTranscript } from "../../../shared/contracts/generated/LocalModelReferenceTranscript";
import type { PreparedVoiceReference } from "../contracts/generated/PreparedVoiceReference";
import type { VoiceCandidateSource } from "../contracts/generated/VoiceCandidateSource";
import type { VoiceLibraryRevision } from "../contracts/generated/VoiceLibraryRevision";
import type { VoiceModel } from "../contracts/generated/VoiceModel";
import { useVoiceCandidates } from "./use-voice-candidates";

export function useVoiceCandidateForm(spaceId: () => string, created: (voice: VoiceLibraryRevision) => void) {
  const { t } = useI18n();
  const mode = shallowRef<"reference" | "designed">("reference");
  const model = shallowRef<VoiceModel>("mossNano");
  const modelAvailable = shallowRef(false);
  const referenceTranscript = shallowRef<LocalModelReferenceTranscript>("unused");
  const language = shallowRef("zh-CN");
  const description = shallowRef("");
  const reference = shallowRef<PreparedVoiceReference>();
  const preparingReference = shallowRef(false);
  const transcript = shallowRef("");
  const transcriptConfirmed = shallowRef(false);
  const authorized = shallowRef(false);
  const text = shallowRef(t("spaces.speech.candidateSample"));
  watch(
    reference,
    (prepared) => {
      transcript.value = prepared?.transcript ?? "";
      transcriptConfirmed.value = false;
    },
    { flush: "sync" },
  );
  const { candidates, activeRunId, pendingRequestId, busy, error, draft, save, cancel, discard, audition } =
    useVoiceCandidates(spaceId, created);
  const modes = computed(() => [
    { value: "reference", label: t("spaces.speech.cloneVoice") },
    { value: "designed", label: t("spaces.speech.designVoice") },
  ]);
  const languages = [
    { value: "zh-CN", label: "中文" },
    { value: "en", label: "English" },
  ];
  const sourceReady = computed(() =>
    mode.value === "designed"
      ? Boolean(description.value.trim())
      : Boolean(reference.value) &&
        !preparingReference.value &&
        (referenceTranscript.value === "unused" || !transcript.value.trim() || transcriptConfirmed.value),
  );
  const ready = computed(
    () => authorized.value && modelAvailable.value && text.value.trim() && sourceReady.value,
  );

  function buildSource(): VoiceCandidateSource | undefined {
    if (mode.value === "designed") {
      return { kind: "designed", description: description.value.trim() };
    }
    const prepared = reference.value;
    if (!prepared) return undefined;
    const usesTranscript = referenceTranscript.value === "optionalConfirmed";
    return {
      kind: "reference",
      sourcePath: prepared.sourcePath,
      sourceRevision: prepared.sourceRevision,
      startMs: prepared.startMs,
      endMs: prepared.endMs,
      referenceText: usesTranscript ? transcript.value.trim() || null : null,
      referenceTextConfirmed: usesTranscript && transcriptConfirmed.value,
    };
  }

  function generate() {
    if (!ready.value || busy.value) return;
    const candidateSource = buildSource();
    if (!candidateSource) return;
    void draft({
      model: model.value,
      language: language.value,
      text: text.value.trim(),
      authorized: authorized.value,
      seed: crypto.getRandomValues(new Uint32Array(1))[0]!,
      source: candidateSource,
    });
  }

  return {
    t,
    mode,
    model,
    modelAvailable,
    referenceTranscript,
    language,
    description,
    reference,
    preparingReference,
    transcript,
    transcriptConfirmed,
    authorized,
    text,
    candidates,
    activeRunId,
    pendingRequestId,
    busy,
    error,
    modes,
    languages,
    ready,
    generate,
    save,
    cancel,
    discard,
    audition,
  };
}
