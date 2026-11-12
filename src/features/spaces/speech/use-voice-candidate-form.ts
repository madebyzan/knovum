import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { LocalModelReferenceDuration } from "../../../shared/contracts/generated/LocalModelReferenceDuration";
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
  // The model select adopts this machine's preferred synthesis model once the catalogue lands;
  // that adoption initializes the form instead of editing it, so the dirty baseline moves with it.
  const initialModel = shallowRef<VoiceModel>(DEFAULT_MODEL);
  const modelAvailable = shallowRef(false);
  const supportsDesign = shallowRef(false);
  const referenceTranscript = shallowRef<LocalModelReferenceTranscript>("unused");
  const referenceDuration = shallowRef<LocalModelReferenceDuration>();
  const language = shallowRef("zh-CN");
  const description = shallowRef("");
  const reference = shallowRef<PreparedVoiceReference>();
  const preparingReference = shallowRef(false);
  const transcript = shallowRef("");
  const transcriptConfirmed = shallowRef(false);
  // Most recordings worth cloning from — films, interviews, videos — carry music or effects.
  const isolateVoice = shallowRef(true);
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
  const referenceDurationValid = computed(() => {
    const prepared = reference.value;
    if (!prepared) return false;
    const duration = prepared.endMs - prepared.startMs;
    const bounds = referenceDuration.value ?? { minMs: 5_000, maxMs: 30_000 };
    return duration >= bounds.minMs && duration <= bounds.maxMs;
  });
  const sourceReady = computed(() =>
    mode.value === "designed"
      ? Boolean(description.value.trim())
      : referenceDurationValid.value &&
        !preparingReference.value &&
        (referenceTranscript.value === "unused" || !transcript.value.trim() || transcriptConfirmed.value),
  );
  const ready = computed(() => modelAvailable.value && text.value.trim() && sourceReady.value);
  const dirty = computed(
    () =>
      mode.value !== initialMode ||
      model.value !== initialModel.value ||
      language.value !== "zh-CN" ||
      Boolean(description.value.trim() || reference.value || transcript.value.trim()) ||
      text.value !== initialText ||
      !isolateVoice.value,
  );

  function buildSource(): VoiceCandidateSource | undefined {
    if (mode.value === "designed") {
      return { kind: "designed", description: description.value.trim() };
    }
    const prepared = reference.value;
    if (!prepared) return undefined;
    const usesTranscript = referenceTranscript.value === "optionalConfirmed";
    const prompt = supportsDesign.value ? description.value.trim() : "";
    return {
      kind: "reference",
      sourcePath: prepared.sourcePath,
      sourceRevision: prepared.sourceRevision,
      startMs: prepared.startMs,
      endMs: prepared.endMs,
      referenceText: usesTranscript ? transcript.value.trim() || null : null,
      referenceTextConfirmed: usesTranscript && transcriptConfirmed.value,
      designPrompt: prompt || null,
      isolateVoice: isolateVoice.value,
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
    initialModel,
    modelAvailable,
    supportsDesign,
    referenceTranscript,
    referenceDuration,
    language,
    description,
    reference,
    preparingReference,
    transcript,
    transcriptConfirmed,
    isolateVoice,
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
