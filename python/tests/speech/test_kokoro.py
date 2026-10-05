from __future__ import annotations

import threading
from pathlib import Path
from types import SimpleNamespace

import pytest

from knovum_inference.contracts.generated.synthesis import SpeechSynthesizeParams
from knovum_inference.errors import RpcError
from knovum_inference.speech.synthesis.engine import SynthesisEngine


def params() -> SpeechSynthesizeParams:
    return SpeechSynthesizeParams.model_validate_json("""{
        "requestId":"synthesis", "attemptEpoch":1, "attemptId":"attempt", "ownerKind":"speech",
        "ownerId":"owner", "inputRevision":"revision", "modelSpecId":"kokoro-model",
        "deadlineMs":9223372036854775807, "text":"Hello. World!", "seed":0,
        "voiceRevision":{
            "sourceKind":"preset", "speakerId":3, "revisionId":"voice",
            "referenceAudio":null, "referenceText":null, "designPrompt":null
        },
        "profile":{"engine":"kokoro", "language":"en-US", "sampleRate":24000, "speed":1.0}
    }""")


class Publisher:
    def __init__(self) -> None:
        self.chunks: list[tuple[object, int, str]] = []

    def publish(self, samples: object, sample_rate: int, granularity: str) -> None:
        self.chunks.append((samples, sample_rate, granularity))

    def finish(self) -> dict[str, object]:
        return {"requestId": "synthesis", "sampleCount": len(self.chunks)}


def test_kokoro_routing_preserves_sentence_stream_and_voice_controls(tmp_path: Path) -> None:
    calls: list[dict[str, object]] = []

    def generate(**controls: object) -> SimpleNamespace:
        calls.append(controls)
        return SimpleNamespace(sample_rate=24000, samples=[0.1, -0.1])

    engine = SynthesisEngine(tmp_path, "cpu", 2)
    engine._kokoro._engine = SimpleNamespace(generate=generate)
    publisher = Publisher()
    response = engine.synthesize(params(), publisher, threading.Event())  # type: ignore[arg-type]
    assert response == {"requestId": "synthesis", "sampleCount": 2, "modelSpecId": "kokoro-model"}
    assert calls == [{"text": "Hello.", "sid": 3, "speed": 1.0}, {"text": "World!", "sid": 3, "speed": 1.0}]
    assert publisher.chunks == [([0.1, -0.1], 24000, "sentence")] * 2


@pytest.mark.parametrize("failure", ["cancelled", "invalid_output"])
def test_kokoro_rejects_cancelled_or_invalid_audio_without_publishing(tmp_path: Path, failure: str) -> None:
    engine = SynthesisEngine(tmp_path, "cpu", 2)
    engine._kokoro._engine = SimpleNamespace(generate=lambda **_: SimpleNamespace(sample_rate=48000, samples=[0.1]))
    publisher = Publisher()
    cancelled = threading.Event()
    if failure == "cancelled":
        cancelled.set()
    with pytest.raises(RpcError) as caught:
        engine.synthesize(params(), publisher, cancelled)  # type: ignore[arg-type]
    assert caught.value.stable_code == failure
    assert publisher.chunks == []
