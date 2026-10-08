from __future__ import annotations

import threading
from pathlib import Path
from types import SimpleNamespace

import numpy as np
import pytest

from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams
from knovum_inference.errors import RpcError
from knovum_inference.speech.synthesis.engine import SynthesisEngine


def params() -> SpeechSynthesizeParams:
    return SpeechSynthesizeParams.model_validate_json("""{
        "requestId":"synthesis", "attemptEpoch":1, "attemptId":"attempt", "ownerKind":"speech",
        "ownerId":"owner", "inputRevision":"revision", "modelSpecId":"moss-tts-nano-onnx-v1",
        "deadlineMs":9223372036854775807, "text":"Hello world!", "seed":0,
        "voiceRevision":{"sourceKind":"preset", "speakerId":0, "revisionId":"voice"},
        "profile":{"engine":"moss_nano", "language":"en", "sampleRate":48000, "speed":1.0}
    }""")


class Publisher:
    def __init__(self) -> None:
        self.chunks: list[tuple[object, int, str]] = []

    def publish(self, samples: object, sample_rate: int, granularity: str) -> None:
        self.chunks.append((samples, sample_rate, granularity))

    def finish(self) -> dict[str, object]:
        return {"requestId": "synthesis", "sampleCount": len(self.chunks)}


def runtime(cancelled: threading.Event | None = None) -> SimpleNamespace:
    def generate(rows: object, *, on_frame: object) -> list[list[int]]:
        frames = [[1] * 16 for _ in range(10)]
        for index, frame in enumerate(frames):
            if index == 1 and cancelled is not None:
                cancelled.set()
            on_frame(rows, index, frame)
        return frames

    return SimpleNamespace(
        list_builtin_voices=lambda: [{"voice": "Junhao", "prompt_audio_codes": [[2] * 16]}],
        build_voice_clone_request_rows=lambda codes, text: {"codes": codes, "text": text},
        generate_audio_frames=generate,
        codec_streaming_session=SimpleNamespace(
            reset=lambda: None,
            run_frames=lambda frames: (np.full((1, 2, len(frames) * 2), 0.2, np.float32), len(frames) * 2),
        ),
    )


def test_moss_publishes_bounded_frames_and_final_tail(tmp_path: Path) -> None:
    engine = SynthesisEngine(tmp_path, "cpu", 2, params().model_spec_id)
    assert engine._moss_nano is not None
    engine._moss_nano._engine = runtime()
    engine._moss_nano._tokenizer = SimpleNamespace(encode=lambda text, out_type: [1, 2])
    publisher = Publisher()
    response = engine.synthesize(params(), publisher, threading.Event())
    assert response["sampleCount"] == 2
    assert [len(chunk[0]) for chunk in publisher.chunks] == [16, 4]
    assert all(chunk[1:] == (48000, "model_chunk") for chunk in publisher.chunks)


def test_moss_cancellation_stops_before_chunk_publication(tmp_path: Path) -> None:
    cancelled = threading.Event()
    engine = SynthesisEngine(tmp_path, "cpu", 2, params().model_spec_id)
    assert engine._moss_nano is not None
    engine._moss_nano._engine = runtime(cancelled)
    engine._moss_nano._tokenizer = SimpleNamespace(encode=lambda text, out_type: [1])
    publisher = Publisher()
    with pytest.raises(RpcError) as caught:
        engine.synthesize(params(), publisher, cancelled)
    assert caught.value.stable_code == "cancelled"
    assert publisher.chunks == []


@pytest.mark.parametrize("control", ["speed", "style", "design", "transcript", "sample_rate"])
def test_moss_rejects_controls_it_cannot_honor(tmp_path: Path, control: str) -> None:
    request = params()
    if control == "speed":
        request.profile.speed = 1.2
    elif control == "style":
        request.style = "cheerful"
    elif control == "design":
        request.voice_revision.design_prompt = "deep voice"
    elif control == "transcript":
        request.voice_revision.reference_text = "Reference words"
    else:
        request.profile.sample_rate = 24000
    with pytest.raises(RpcError):
        SynthesisEngine(tmp_path, "cpu", 2, request.model_spec_id).synthesize(request, Publisher(), threading.Event())


def test_moss_rejects_exhausted_generation_instead_of_publishing_truncated_tail(tmp_path: Path) -> None:
    engine = SynthesisEngine(tmp_path, "cpu", 2, params().model_spec_id)
    assert engine._moss_nano is not None
    engine._moss_nano._engine = runtime()
    engine._moss_nano._engine.generate_audio_frames = lambda rows, on_frame: [[1] * 16] * 1200
    engine._moss_nano._tokenizer = SimpleNamespace(encode=lambda text, out_type: [1])
    publisher = Publisher()
    with pytest.raises(RpcError) as caught:
        engine.synthesize(params(), publisher, threading.Event())
    assert caught.value.stable_code == "resource_exhausted"
    assert publisher.chunks == []
