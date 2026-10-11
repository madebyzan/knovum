import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { LocalModelReferenceTranscript } from "../../../shared/contracts/generated/LocalModelReferenceTranscript";
import type { PreparedVoiceReference } from "../contracts/generated/PreparedVoiceReference";
import type { VoiceCandidateSource } from "../contracts/generated/VoiceCandidateSource";
import type { VoiceModel } from "../contracts/generated/VoiceModel";
import { useVoiceCandidates } from "./use-voice-candidates";

export type VoiceCandidateSourceKind = "reference" | "designed";
const DEFAULT_MODEL: VoiceModel = "mossNano";

export function useVoiceCandidateForm(requestedSourceKind?: () => VoiceCandidateSourceKind | undefined) {
  const { t } = useI18n();
  const initialMode = requestedSourceKind?.() ?? "reference";
  const mode = shallowRef<VoiceCandidateSourceKind>(initialMode);
  const model = shallowRef<VoiceModel>(DEFAULT_MODEL);
  const modelAvailable = shallowRef(false);
  const referenceTranscript = shallowRef<LocalModelReferenceTranscript>("unused");
  const language = shallowRef("zh-CN");
  const description = shallowRef("");
  const reference = shallowRef<PreparedVoiceReference>();
  const preparingReference = shallowRef(false);
  const transcript = shallowRef("");
  const transcriptConfirmed = shallowRef(false);
  const initialText = t("spaces.speech.voice.candidate.sample");
  const text = shallowRef(initialText);
  watch(
    reference,
    (prepared) => {
      transcript.value = prepared?.transcript ?? "";
      transcriptConfirmed.value = false;
    },
    { flush: "sync" },
  );
  const {
    candidates,
    activeRunId,
    pendingRequestId,
    busy,
    isCancelling,
    errors,
    draft,
    save,
    cancel,
    discard,
    audition,
  } = useVoiceCandidates();
  if (requestedSourceKind)
    watch(requestedSourceKind, (sourceKind) => {
      if (sourceKind) mode.value = sourceKind;
    });
  const modes = computed(() => [
    { value: "reference", label: t("spaces.speech.voice.clone.fromReference") },
    { value: "designed", label: t("spaces.speech.voice.design.action") },
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
  const ready = computed(() => modelAvailable.value && text.value.trim() && sourceReady.value);
  const dirty = computed(
    () =>
      mode.value !== initialMode ||
      model.value !== DEFAULT_MODEL ||
      language.value !== "zh-CN" ||
      Boolean(description.value.trim() || reference.value || transcript.value.trim()) ||
      text.value !== initialText,
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
      seed: crypto.getRandomValues(new Uint32Array(1))[0]!,
      source: candidateSource,
    });
  }

  return {
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
    text,
    candidates,
    activeRunId,
    pendingRequestId,
    busy,
    isCancelling,
    errors,
    modes,
    languages,
    ready,
    dirty,
    generate,
    save,
    cancel,
    discard,
    audition,
  };
}
