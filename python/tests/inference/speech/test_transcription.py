from __future__ import annotations

import hashlib
import json
from pathlib import Path
from threading import Event
from uuid import uuid4

import pytest

from knovum_inference.contracts.generated.transcription import AsrEngine, LanguagePolicy, SpeechTranscribeParams
from knovum_inference.errors import RpcError
from knovum_inference.speech.transcription import TranscriptionEngine


def _params(request_id: str, audio_ref: dict[str, object] | None = None) -> SpeechTranscribeParams:
    return SpeechTranscribeParams.model_validate(
        {
            "requestId": request_id,
            "attemptEpoch": 7,
            "attemptId": str(uuid4()),
            "audioRef": audio_ref
            or {
                "path": "/unused.wav",
                "mediaType": "audio/wav",
                "byteLength": 0,
                "sha256": hashlib.sha256(b"").hexdigest(),
                "sampleRate": 16_000,
                "channels": 1,
                "sampleFormat": "pcm_s16le",
            },
            "deadlineMs": 2**63,
            "inputRevision": "source-revision",
            "modelSpecId": "asr-model-v1",
            "ownerId": "media-node",
            "ownerKind": "media_transcript",
            "profile": {
                "engine": AsrEngine.faster_whisper,
                "languagePolicy": LanguagePolicy.detect,
                "language": None,
                "device": "cpu",
                "computeType": "int8",
                "beamSize": 5,
                "vadEnabled": True,
            },
        }
    )


def _artifact() -> dict[str, object]:
    return {
        "language": "en",
        "durationMs": 1_500,
        "speechDurationMs": 900,
        "segments": [
            {
                "startMs": 100,
                "endMs": 1_000,
                "text": "hello",
                "words": [
                    {
                        "startMs": 100,
                        "endMs": 500,
                        "text": "hello",
                        "probability": 0.95,
                    }
                ],
            }
        ],
    }


def test_publish_validates_generated_artifact_and_preserves_response_identity(tmp_path: Path) -> None:
    request_id = str(uuid4())
    params = _params(request_id)
    result = TranscriptionEngine(tmp_path / "model", tmp_path, 2)._publish(params, _artifact())

    transcript_ref = result["transcriptRef"]
    artifact_path = tmp_path / transcript_ref["fileName"]
    payload = artifact_path.read_bytes()
    assert json.loads(payload)["speechDurationMs"] == 900
    assert result["requestId"] == request_id
    assert result["attemptEpoch"] == 7
    assert result["modelSpecId"] == "asr-model-v1"
    assert transcript_ref["byteLength"] == len(payload)
    assert transcript_ref["sha256"] == hashlib.sha256(payload).hexdigest()
    assert list(artifact_path.parent.iterdir()) == [artifact_path]


@pytest.mark.parametrize(
    "change",
    [
        {"speechDurationMs": -1},
        {"unexpected": "field"},
    ],
)
def test_publish_rejects_artifacts_outside_generated_contract(tmp_path: Path, change: dict[str, object]) -> None:
    artifact = _artifact() | change
    with pytest.raises(ValueError):
        TranscriptionEngine(tmp_path / "model", tmp_path, 2)._publish(_params(str(uuid4())), artifact)
    assert list(tmp_path.iterdir()) == []


def test_publish_rejects_unsafe_attempt_identity_before_writing(tmp_path: Path) -> None:
    with pytest.raises(RpcError) as caught:
        TranscriptionEngine(tmp_path / "model", tmp_path, 2)._publish(_params("../outside"), _artifact())
    assert caught.value.stable_code == "invalid_output"
    assert list(tmp_path.iterdir()) == []


def test_cancelled_transcription_never_publishes_completed_artifact(tmp_path: Path) -> None:
    audio_path = tmp_path / "source.wav"
    audio_path.write_bytes(b"source")
    audio_ref = {
        "path": str(audio_path),
        "mediaType": "audio/wav",
        "byteLength": len(b"source"),
        "sha256": hashlib.sha256(b"source").hexdigest(),
        "sampleRate": 16_000,
        "channels": 1,
        "sampleFormat": "pcm_s16le",
    }
    engine = TranscriptionEngine(tmp_path / "model", tmp_path, 2)
    engine._faster = lambda *_: _artifact()  # type: ignore[method-assign]
    cancelled = Event()
    cancelled.set()

    with pytest.raises(RpcError) as caught:
        engine.transcribe(
            _params(str(uuid4()), audio_ref),
            lambda _stage, _percent, _language, _partial: None,
            cancelled,
        )
    assert caught.value.stable_code == "cancelled"
    assert not any(path.name.startswith("attempt-") for path in tmp_path.iterdir())
