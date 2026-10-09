from __future__ import annotations

import hashlib
import json
import os
import platform
import sys
import types
from pathlib import Path
from threading import Event
from uuid import uuid4

import pytest

from knovum_inference.contracts.generated.control import RuntimeDeviceProfile
from knovum_inference.contracts.generated.transcription import AsrEngine, LanguagePolicy, SpeechTranscribeParams
from knovum_inference.errors import RpcError
from knovum_inference.speech.transcription import TranscriptionEngine

PROFILE_ID = "faster-whisper-small-cpu-int8-v1"


def _device_profile(threads: int = 2) -> RuntimeDeviceProfile:
    return RuntimeDeviceProfile.model_validate(
        {"device": "cpu", "executionProfileId": PROFILE_ID, "cpuThreads": threads}
    )


def _engine(model_directory: Path, work_directory: Path, threads: int = 2) -> TranscriptionEngine:
    return TranscriptionEngine(model_directory, work_directory, _device_profile(threads))


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
            "modelSpecId": PROFILE_ID,
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
    publication = _engine(tmp_path / "model", tmp_path)._publish(params, _artifact())

    transcript_ref = publication["transcriptRef"]
    artifact_path = tmp_path / transcript_ref["fileName"]
    payload = artifact_path.read_bytes()
    assert json.loads(payload)["speechDurationMs"] == 900
    assert publication["requestId"] == request_id
    assert publication["attemptEpoch"] == 7
    assert publication["modelSpecId"] == PROFILE_ID
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
        _engine(tmp_path / "model", tmp_path)._publish(_params(str(uuid4())), artifact)
    assert list(tmp_path.iterdir()) == []


def test_publish_rejects_unsafe_attempt_identity_before_writing(tmp_path: Path) -> None:
    with pytest.raises(RpcError) as caught:
        _engine(tmp_path / "model", tmp_path)._publish(_params("../outside"), _artifact())
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
    engine = _engine(tmp_path / "model", tmp_path)
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


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("model_spec_id", "faster-whisper-small-cuda-f16-v1"),
        ("engine", AsrEngine.mlx_whisper),
        ("device", "cuda"),
        ("compute_type", "float16"),
    ],
)
def test_transcription_rejects_request_execution_identity_before_reading_audio(
    tmp_path: Path, field: str, value: object
) -> None:
    params = _params(str(uuid4()))
    if field == "model_spec_id":
        params.model_spec_id = str(value)
    else:
        setattr(params.profile, field, value)

    with pytest.raises(RpcError) as caught:
        _engine(tmp_path / "model", tmp_path).transcribe(params, lambda *_: None, Event())

    assert caught.value.stable_code == "runtime_mismatch"


@pytest.mark.parametrize(
    ("device", "profile_id"),
    [
        ("cpu", "faster-whisper-small-cpu-f16-v1"),
        ("cpu", "faster-whisper-arbitrary-cpu-int8-v1"),
        ("cuda", "faster-whisper-small-cuda-int8-v1"),
        ("mps", "faster-whisper-small-mps-f16-v1"),
    ],
)
def test_transcription_rejects_unknown_admitted_execution_profiles(
    tmp_path: Path, device: str, profile_id: str
) -> None:
    profile = RuntimeDeviceProfile.model_validate({"device": device, "executionProfileId": profile_id, "cpuThreads": 2})
    with pytest.raises(RpcError) as caught:
        TranscriptionEngine(tmp_path / "model", tmp_path, profile)
    assert caught.value.stable_code == "runtime_mismatch"


@pytest.mark.parametrize(("system", "machine"), [("Darwin", "x86_64"), ("Windows", "AMD64")])
def test_cpu_transcription_uses_compiled_faster_whisper(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path, system: str, machine: str
) -> None:
    calls: dict[str, object] = {}

    class WhisperModel:
        def __init__(self, path: str, **options: object) -> None:
            calls.update(path=path, **options)

        def transcribe(self, path: str, **options: object) -> tuple[list[object], object]:
            return [], types.SimpleNamespace(language="en", duration=1.0, duration_after_vad=0.8)

    monkeypatch.setitem(sys.modules, "faster_whisper", types.SimpleNamespace(WhisperModel=WhisperModel))
    monkeypatch.setattr(platform, "system", lambda: system)
    monkeypatch.setattr(platform, "machine", lambda: machine)
    monkeypatch.setenv("OMP_NUM_THREADS", "session-owned")
    artifact = _engine(tmp_path, tmp_path)._faster(
        tmp_path / "source.wav", _params(str(uuid4())), lambda *_: None, Event()
    )
    assert calls == {
        "path": str(tmp_path),
        "device": "cpu",
        "compute_type": "int8",
        "cpu_threads": 2,
        "local_files_only": True,
    }
    assert artifact["durationMs"] == 1000
    assert os.environ["OMP_NUM_THREADS"] == "session-owned"


def test_intel_transcription_rejects_gpu_profile(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.setattr(platform, "system", lambda: "Darwin")
    monkeypatch.setattr(platform, "machine", lambda: "x86_64")
    params = _params(str(uuid4()))
    params.profile.device = "cuda"
    with pytest.raises(RpcError, match="Intel macOS ASR requires the CPU profile"):
        _engine(tmp_path, tmp_path)._faster(tmp_path / "source.wav", params, lambda *_: None, Event())
