from __future__ import annotations

import threading
from pathlib import Path
from typing import Any

from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams, VoiceSourceKind
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline
from knovum_inference.speech.synthesis.stream import SynthesisChunkPublisher


class KokoroSynthesisEngine:
    def __init__(self, model_directory: Path, device: str, cpu_threads: int) -> None:
        self._model_directory = model_directory
        self._device = device
        self._cpu_threads = cpu_threads
        self._engine: Any = None

    def generate(
        self, params: SpeechSynthesizeParams, publisher: SynthesisChunkPublisher, cancelled: threading.Event
    ) -> None:
        if self._device != "cpu" or params.voice_revision.source_kind is not VoiceSourceKind.preset:
            raise invalid_params("Kokoro requires its CPU preset synthesis profile")
        if (
            params.voice_revision.speaker_id is None
            or params.voice_revision.reference_audio is not None
            or params.voice_revision.reference_text is not None
            or params.voice_revision.design_prompt is not None
            or params.style is not None
            or params.seed != 0
        ):
            raise invalid_params("Kokoro preset voice controls are invalid")
        if not 0.5 <= params.profile.speed <= 2.0:
            raise invalid_params("Kokoro synthesis speed is invalid")
        for sentence in _sentences(params.text):
            if cancelled.is_set():
                raise server_error("cancelled", "Synthesis was cancelled")
            check_deadline(params.deadline_ms)
            audio = self._load().generate(
                text=sentence,
                sid=params.voice_revision.speaker_id,
                speed=params.profile.speed,
            )
            sample_rate = int(audio.sample_rate)
            if sample_rate != params.profile.sample_rate:
                raise server_error("invalid_output", "Synthesis sentence output is invalid")
            publisher.publish(audio.samples, sample_rate, "sentence")

    def _load(self) -> Any:
        if self._engine is not None:
            return self._engine
        try:
            import sherpa_onnx
        except ImportError as error:
            raise server_error("runtime_mismatch", "sherpa-onnx is unavailable") from error
        model = self._model_directory / "model.int8.onnx"
        voices = self._model_directory / "voices.bin"
        tokens = self._model_directory / "tokens.txt"
        data_dir = self._model_directory / "espeak-ng-data"
        dictionary = self._model_directory / "dict"
        lexicons = [
            self._model_directory / name for name in ("lexicon-us-en.txt", "lexicon-gb-en.txt", "lexicon-zh.txt")
        ]
        rule_fsts = [self._model_directory / name for name in ("phone-zh.fst", "date-zh.fst", "number-zh.fst")]
        if not all(path.exists() for path in (model, voices, tokens, data_dir, dictionary, *lexicons, *rule_fsts)):
            raise server_error("model_corrupt", "Kokoro model snapshot is incomplete")
        kokoro = sherpa_onnx.OfflineTtsKokoroModelConfig(
            model=str(model),
            voices=str(voices),
            tokens=str(tokens),
            data_dir=str(data_dir),
            dict_dir=str(dictionary),
            lexicon=",".join(map(str, lexicons)),
        )
        config = sherpa_onnx.OfflineTtsModelConfig(
            kokoro=kokoro, num_threads=self._cpu_threads, debug=False, provider="cpu"
        )
        self._engine = sherpa_onnx.OfflineTts(
            sherpa_onnx.OfflineTtsConfig(model=config, rule_fsts=",".join(map(str, rule_fsts)))
        )
        return self._engine


def _sentences(text: str) -> list[str]:
    sentences: list[str] = []
    start = 0
    for index, character in enumerate(text, start=1):
        if character in ".!?。！？；;\n" or index - start >= 500:
            sentence = text[start:index].strip()
            if sentence:
                sentences.append(sentence)
            start = index
    tail = text[start:].strip()
    if tail:
        sentences.append(tail)
    return sentences
