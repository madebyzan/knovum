from __future__ import annotations

import threading
from pathlib import Path
from typing import Any

from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams, VoiceSourceKind
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.speech.request import check_deadline
from knovum_inference.speech.synthesis.moss_reference import reference_codes
from knovum_inference.speech.synthesis.stream import SynthesisChunkPublisher

SAMPLE_RATE = 48_000
MAX_FRAMES = 1_200
CHUNK_FRAMES = 8
PRESET_VOICES = ("Junhao", "Zhiming", "Weiguo", "Xiaoyu", "Yuewen", "Lingyu", "Ava", "Bella", "Adam", "Nathan")


class MossNanoSynthesisEngine:
    def __init__(self, model_directory: Path, device: str, cpu_threads: int) -> None:
        self._model_directory = model_directory
        self._device = device
        self._cpu_threads = cpu_threads
        self._engine: Any = None
        self._tokenizer: Any = None

    def generate(
        self, params: SpeechSynthesizeParams, publisher: SynthesisChunkPublisher, cancelled: threading.Event
    ) -> None:
        import numpy as np

        if params.profile.sample_rate != SAMPLE_RATE or params.profile.speed != 1.0:
            raise invalid_params("MOSS-TTS-Nano requires 48 kHz output at normal speed")
        if params.style is not None or params.voice_revision.design_prompt is not None:
            raise invalid_params("MOSS-TTS-Nano does not support voice design or style guidance")
        if params.voice_revision.reference_text is not None:
            raise invalid_params("MOSS-TTS-Nano does not support transcript conditioning")
        self._check(params, cancelled)
        engine = self._load()
        codes = self._prompt_codes(params)
        text = self._tokenizer.encode(params.text, out_type=int)
        engine.rng = np.random.default_rng(params.seed)
        rows = engine.build_voice_clone_request_rows(codes, text)
        engine.codec_streaming_session.reset()
        pending: list[list[int]] = []

        def flush() -> None:
            self._check(params, cancelled)
            decoded = engine.codec_streaming_session.run_frames(pending[:])
            pending.clear()
            if decoded is not None:
                audio, count = decoded
                if count:
                    # Knovum's streaming/output contract is mono. Preserve both supplier channels.
                    publisher.publish(np.mean(audio[0, :, :count], axis=0), SAMPLE_RATE, "model_chunk")

        def on_frame(_frames: list[list[int]], _index: int, frame: list[int]) -> None:
            self._check(params, cancelled)
            pending.append(frame)
            if len(pending) >= CHUNK_FRAMES:
                flush()

        frames = engine.generate_audio_frames(rows, on_frame=on_frame)
        self._check(params, cancelled)
        if len(frames) == MAX_FRAMES:
            raise server_error("resource_exhausted", "Speech generation reached its duration limit")
        if pending:
            flush()

    @staticmethod
    def _check(params: SpeechSynthesizeParams, cancelled: threading.Event) -> None:
        if cancelled.is_set():
            raise server_error("cancelled", "Synthesis was cancelled")
        check_deadline(params.deadline_ms)

    def _prompt_codes(self, params: SpeechSynthesizeParams) -> list[list[int]]:
        revision = params.voice_revision
        if revision.source_kind is VoiceSourceKind.reference:
            if revision.speaker_id is not None or revision.reference_audio is None:
                raise invalid_params("Reference cloning requires an immutable audio revision")
            return reference_codes(self._engine, revision.reference_audio)
        if (
            revision.source_kind is not VoiceSourceKind.preset
            or revision.speaker_id is None
            or not 0 <= revision.speaker_id < len(PRESET_VOICES)
            or revision.reference_audio is not None
            or revision.reference_text is not None
        ):
            raise invalid_params("MOSS-TTS-Nano preset controls are invalid")
        selected = PRESET_VOICES[revision.speaker_id]
        for voice in self._engine.list_builtin_voices():
            if voice["voice"] == selected:
                return voice["prompt_audio_codes"]  # type: ignore[no-any-return]
        raise server_error("model_corrupt", "Preset voice is absent from the verified model")

    def _load(self) -> Any:
        if self._engine is not None:
            return self._engine
        if self._device not in {"cpu", "cuda"}:
            raise server_error("unsupported_device", "MOSS ONNX requires a supported execution provider")
        try:
            import sentencepiece
            from moss_tts_nano_ort.ort_cpu_runtime import OrtCpuRuntime
        except ImportError as error:
            raise server_error("runtime_mismatch", "MOSS-TTS-Nano runtime is unavailable") from error

        # The pinned standalone supplier is installed only in the managed inference runtime.
        class InstalledRuntime(OrtCpuRuntime):  # type: ignore[misc]
            model_dir: Path

            # The app's verified snapshot contains both suppliers in one directory. No aliases,
            # external paths or compatibility searches are part of Knovum's model contract.
            def resolve_manifest_relative_path(self, relative_path: str | Path) -> Path:
                files = {
                    "tts_browser_onnx_meta.json": "tts_browser_onnx_meta.json",
                    "../MOSS-Audio-Tokenizer-Nano-ONNX/codec_browser_onnx_meta.json": "codec_browser_onnx_meta.json",
                    "tokenizer.model": "tokenizer.model",
                }
                return self.model_dir / files[str(relative_path)]

        self._engine = InstalledRuntime(
            self._model_directory,
            thread_count=self._cpu_threads,
            max_new_frames=MAX_FRAMES,
            execution_provider=self._device,
        )
        self._tokenizer = sentencepiece.SentencePieceProcessor(
            model_file=str(self._model_directory / "tokenizer.model")
        )
        return self._engine
