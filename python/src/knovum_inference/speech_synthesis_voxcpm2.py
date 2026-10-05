from __future__ import annotations

import platform
import threading
import wave
from pathlib import Path
from typing import Any

from knovum_inference.contracts import AudioFileReference, SpeechSynthesizeParams, VoiceSourceKind
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech_request import verify_audio
from knovum_inference.speech_stream import SynthesisChunkPublisher

SAMPLE_RATE = 48_000
_REQUIRED_FILES = (
    "audiovae.pth",
    "config.json",
    "model.safetensors",
    "special_tokens_map.json",
    "tokenization_voxcpm2.py",
    "tokenizer.json",
    "tokenizer_config.json",
)


class VoxCpm2SynthesisEngine:
    def __init__(self, model_directory: Path, device: str) -> None:
        self._model_directory = model_directory
        self._device = device
        self._engine: Any = None
        self._vad: Any = None

    def generate(
        self,
        params: SpeechSynthesizeParams,
        publisher: SynthesisChunkPublisher,
        cancelled: threading.Event,
    ) -> None:
        system = platform.system()
        machine = platform.machine().lower()
        qualified = (system == "Darwin" and machine == "arm64" and self._device == "mps") or (
            system == "Windows" and machine in {"amd64", "x86_64"} and self._device == "cpu"
        )
        if not qualified:
            raise server_error("unsupported_device", "VoxCPM2 device profile is not qualified")
        if params.profile.sample_rate != SAMPLE_RATE or params.profile.speed != 1.0:
            raise invalid_params("VoxCPM2 requires the fixed 48 kHz profile at normal speed")
        text, prompt_path, prompt_text, reference_path = self._generation_inputs(params)
        try:
            import torch
        except ImportError as error:
            raise server_error("runtime_mismatch", "PyTorch is unavailable") from error
        torch.manual_seed(params.seed)
        engine = self._load()
        sample_rate = int(engine.tts_model.sample_rate)
        if sample_rate != SAMPLE_RATE:
            raise server_error("invalid_output", "VoxCPM2 returned an unexpected sample rate")
        chunks = engine.generate_streaming(
            text=text,
            prompt_wav_path=prompt_path,
            prompt_text=prompt_text,
            reference_wav_path=reference_path,
            normalize=False,
            denoise=False,
            retry_badcase=False,
            max_len=4096,
        )
        try:
            for waveform in chunks:
                if cancelled.is_set():
                    raise server_error("cancelled", "Synthesis was cancelled")
                publisher.publish(waveform.reshape(-1), sample_rate, "model_chunk")
        finally:
            chunks.close()

    def _generation_inputs(self, params: SpeechSynthesizeParams) -> tuple[str, str | None, str | None, str | None]:
        revision = params.voice_revision
        if revision.speaker_id is not None:
            raise invalid_params("VoxCPM2 does not accept a preset speaker ID")
        if revision.source_kind is VoiceSourceKind.designed:
            if (
                revision.reference_audio is not None
                or revision.reference_text is not None
                or revision.design_prompt is None
                or not revision.design_prompt.strip()
                or params.style is not None
            ):
                raise invalid_params("VoxCPM2 voice design controls are invalid")
            return f"({revision.design_prompt.strip()}){params.text}", None, None, None
        if revision.source_kind is not VoiceSourceKind.reference or revision.reference_audio is None:
            raise invalid_params("VoxCPM2 cloning requires an immutable reference audio revision")
        reference = str(self._verify_reference(revision.reference_audio))
        if revision.reference_text is not None:
            if not revision.reference_text.strip() or params.style is not None:
                raise invalid_params("Exact-transcript cloning cannot include style guidance")
            return params.text, reference, revision.reference_text, reference
        controls = [
            value.strip() for value in (revision.design_prompt, params.style) if value is not None and value.strip()
        ]
        text = f"({', '.join(controls)}){params.text}" if controls else params.text
        return text, None, None, reference

    def _verify_reference(self, audio_ref: AudioFileReference) -> Path:
        if (
            audio_ref.media_type != "audio/wav"
            or audio_ref.sample_rate != 16_000
            or audio_ref.channels != 1
            or audio_ref.sample_format != "pcm_s16le"
        ):
            raise server_error("invalid_reference_audio", "Voice reference must be mono PCM16 WAV at 16 kHz")
        path = verify_audio(audio_ref)
        try:
            with wave.open(str(path), "rb") as source:
                if (
                    source.getnchannels() != 1
                    or source.getsampwidth() != 2
                    or source.getframerate() != 16_000
                    or source.getcomptype() != "NONE"
                ):
                    raise server_error("invalid_reference_audio", "Voice reference WAV metadata is invalid")
                frames = source.getnframes()
                if not 5 * 16_000 <= frames <= 30 * 16_000:
                    raise server_error("invalid_reference_audio", "Voice reference must be between 5 and 30 seconds")
                pcm = source.readframes(frames)
        except (OSError, wave.Error) as error:
            raise server_error("invalid_reference_audio", "Voice reference WAV cannot be read") from error
        if len(pcm) != frames * 2:
            raise server_error("invalid_reference_audio", "Voice reference contains truncated PCM frames")
        try:
            import numpy as np
            import torch
            from silero_vad import get_speech_timestamps, load_silero_vad
        except ImportError as error:
            raise server_error("runtime_mismatch", "Silero VAD is unavailable") from error
        if self._vad is None:
            self._vad = load_silero_vad(onnx=True)
        samples = torch.from_numpy(np.frombuffer(pcm, dtype="<i2").astype(np.float32) / 32768.0)
        ranges = get_speech_timestamps(samples, self._vad, sampling_rate=16_000)
        speech_frames = sum(max(0, int(item["end"]) - int(item["start"])) for item in ranges)
        if speech_frames / frames < 0.6:
            raise server_error("invalid_reference_audio", "Voice reference must contain at least 60% speech")
        return path

    def _load(self) -> Any:
        if self._engine is not None:
            return self._engine
        if not all((self._model_directory / name).is_file() for name in _REQUIRED_FILES):
            raise server_error("model_corrupt", "VoxCPM2 model snapshot is incomplete")
        try:
            from voxcpm import VoxCPM
        except ImportError as error:
            raise server_error("runtime_mismatch", "VoxCPM 2.0.3 is unavailable") from error
        engine = VoxCPM.from_pretrained(
            str(self._model_directory),
            device=self._device,
            load_denoiser=False,
            optimize=False,
            local_files_only=True,
        )
        expected_dtype = "float32" if self._device == "mps" else "bfloat16"
        actual_dtype = str(engine.tts_model.config.dtype).lower()
        if actual_dtype != expected_dtype:
            raise server_error(
                "runtime_mismatch",
                f"VoxCPM2 loaded {actual_dtype}, expected frozen {expected_dtype} profile",
            )
        self._engine = engine
        return self._engine
