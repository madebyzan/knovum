from __future__ import annotations

import hashlib
import math
import os
import struct
import wave
from pathlib import Path
from typing import Any

from knovum_inference.contracts import SpeechSynthesisEngine, SpeechSynthesizeParams, VoiceSourceKind
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech_common import attempt_directory, check_deadline

MAX_AUDIO_BYTES = 16 * 1024 * 1024


class SynthesisEngine:
    def __init__(self, model_directory: Path, work_directory: Path, cpu_threads: int) -> None:
        self._model_directory = model_directory
        self._work_directory = work_directory
        self._cpu_threads = cpu_threads
        self._engine: Any = None

    def synthesize(self, params: SpeechSynthesizeParams) -> dict[str, Any]:
        if (
            params.profile.engine is not SpeechSynthesisEngine.kokoro
            or params.voice_revision.source_kind is not VoiceSourceKind.preset
        ):
            raise invalid_params("Kokoro requires its preset synthesis profile")
        if (
            params.voice_revision.speaker_id is None
            or params.voice_revision.reference_audio is not None
            or params.voice_revision.reference_text is not None
            or params.voice_revision.design_prompt is not None
            or params.style is not None
            or params.seed != 0
        ):
            raise invalid_params("Kokoro preset voice controls are invalid")
        if not params.text.strip() or len(params.text) > 10_000 or not 0.5 <= params.profile.speed <= 2.0:
            raise invalid_params("Synthesis text or speed is invalid")
        check_deadline(params.deadline_ms)
        audio = self._load().generate(
            text=params.text, sid=params.voice_revision.speaker_id, speed=params.profile.speed
        )
        sample_rate = int(audio.sample_rate)
        if sample_rate <= 0 or sample_rate != params.profile.sample_rate:
            raise server_error("invalid_output", "Synthesis sample rate differs from the profile")
        samples = [float(sample) for sample in audio.samples]
        if not samples or any(not math.isfinite(sample) for sample in samples):
            raise server_error("invalid_output", "Synthesis returned empty or non-finite audio")
        if len(samples) * 2 + 44 > MAX_AUDIO_BYTES:
            raise server_error("resource_exhausted", "Synthesized audio exceeds its byte limit")
        directory = attempt_directory(self._work_directory, params.request_id)
        file_name = "audio.wav"
        final = directory / file_name
        temporary = directory / f".{file_name}.part"
        _write_wave(temporary, samples, sample_rate)
        os.replace(temporary, final)
        data = final.read_bytes()
        return {
            "requestId": params.request_id,
            "attemptEpoch": params.attempt_epoch,
            "modelSpecId": params.model_spec_id,
            "audioRef": {
                "fileName": f"{directory.name}/{file_name}",
                "mediaType": "audio/wav",
                "byteLength": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
                "sampleRate": sample_rate,
                "channels": 1,
                "sampleFormat": "pcm_s16le",
            },
            "sampleCount": len(samples),
        }

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


def _write_wave(path: Path, samples: list[float], sample_rate: int) -> None:
    with wave.open(str(path), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(sample_rate)
        output.writeframes(
            b"".join(struct.pack("<h", round(max(-1.0, min(1.0, sample)) * 32767)) for sample in samples)
        )
