from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, TypeAlias

from knovum_inference import PROTOCOL_VERSION
from knovum_inference.contracts.generated.control import RuntimeCapability, RuntimeInitializeParams
from knovum_inference.embedding import EmbeddingEngine
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.runtime.identity import verify_runtime_digest
from knovum_inference.speech.alignment import AlignmentEngine
from knovum_inference.speech.diarization import DiarizationEngine
from knovum_inference.speech.synthesis.engine import SynthesisEngine
from knovum_inference.speech.synthesis.voxcpm2 import (
    FULL_MODEL_SPEC_ID,
    Q8_MODEL_SPEC_ID,
)
from knovum_inference.speech.transcription import TranscriptionEngine

if TYPE_CHECKING:
    from knovum_inference.image_segmentation import ImageSegmentationEngine
    from knovum_inference.speech.separation import SpleeterSourceSeparator

    InferenceEngine: TypeAlias = (
        EmbeddingEngine
        | TranscriptionEngine
        | AlignmentEngine
        | DiarizationEngine
        | SynthesisEngine
        | SpleeterSourceSeparator
        | ImageSegmentationEngine
    )

_TORCH_EXECUTION_PROFILES = frozenset(
    {
        "mlx-whisper-small-mps-v1",
        "mlx-whisper-tiny-mps-v1",
        "mlx-whisper-large-v3-mps-v1",
    }
)


@dataclass(slots=True)
class Session:
    session_id: str
    capability: RuntimeCapability
    capability_spec_id: str
    engine: InferenceEngine
    work_directory: Path


def prepare_session(params: RuntimeInitializeParams) -> Session:
    if params.protocol_version != PROTOCOL_VERSION:
        raise server_error("runtime_mismatch", "Protocol version does not match")
    if not params.session_id or not params.capability_spec_id or not params.device_profile.execution_profile_id:
        raise invalid_params("Runtime identity fields must be non-empty")
    if not 1 <= params.device_profile.cpu_threads <= 4:
        raise server_error("unsupported_device", "CPU thread budget must be between one and four")
    work_directory = Path(params.work_directory)
    if not work_directory.is_absolute() or not work_directory.is_dir():
        raise invalid_params("Work directory must be an existing absolute directory")
    verify_runtime_digest(params.runtime_digest, Path(params.runtime_manifest_path))
    model_path = _selected_model_path(params)
    _configure_engine_threads(params)
    engine = _create_engine(params, model_path, work_directory)
    return Session(
        params.session_id,
        params.capability,
        params.capability_spec_id,
        engine,
        work_directory.resolve(strict=True),
    )


def _configure_engine_threads(params: RuntimeInitializeParams) -> None:
    threads = str(params.device_profile.cpu_threads)
    for name in (
        "OMP_NUM_THREADS",
        "MKL_NUM_THREADS",
        "OPENBLAS_NUM_THREADS",
        "NUMEXPR_NUM_THREADS",
        "VECLIB_MAXIMUM_THREADS",
    ):
        os.environ[name] = threads
    uses_torch = (
        params.capability is RuntimeCapability.embedding
        or (
            params.capability is RuntimeCapability.speech_alignment
            and params.capability_spec_id == "qwen3-forced-aligner-0.6b-hf-v1"
        )
        or params.device_profile.execution_profile_id in _TORCH_EXECUTION_PROFILES
    )
    if not uses_torch:
        return
    try:
        import torch
    except ImportError as error:
        raise server_error("runtime_mismatch", "PyTorch is unavailable") from error
    try:
        torch.set_num_threads(params.device_profile.cpu_threads)
        torch.set_num_interop_threads(1)
    except RuntimeError as error:
        raise server_error("runtime_mismatch", "PyTorch thread budget cannot be applied") from error


def _selected_model_path(params: RuntimeInitializeParams) -> Path:
    fields = {
        RuntimeCapability.embedding: params.model_paths.embedding,
        RuntimeCapability.speech_transcription: params.model_paths.speech_transcription,
        RuntimeCapability.speech_alignment: params.model_paths.speech_alignment,
        RuntimeCapability.speech_diarization: params.model_paths.speech_diarization,
        RuntimeCapability.speech_synthesis: params.model_paths.speech_synthesis,
        RuntimeCapability.speech_separation: params.model_paths.speech_separation,
        RuntimeCapability.image_segmentation: params.model_paths.image_segmentation,
    }
    selected = fields[params.capability]
    if selected is None or sum(value is not None for value in fields.values()) != 1:
        raise invalid_params("Exactly one capability model path is required")
    model_path = Path(selected)
    if not model_path.is_absolute() or not model_path.exists():
        raise server_error("model_missing", "Capability model path is missing")
    return model_path.resolve(strict=True)


def _create_engine(params: RuntimeInitializeParams, model_path: Path, work_directory: Path) -> InferenceEngine:
    capability = params.capability
    device = params.device_profile.device
    threads = params.device_profile.cpu_threads
    if capability is RuntimeCapability.embedding:
        if device not in {"cpu", "mps", "cuda"} or not model_path.is_dir():
            raise server_error("unsupported_device", "Embedding profile is unsupported")
        return EmbeddingEngine(str(model_path), device, threads)
    if capability is RuntimeCapability.speech_transcription:
        if not model_path.is_dir():
            raise server_error("model_missing", "ASR model directory is missing")
        return TranscriptionEngine(model_path, work_directory, threads)
    if capability is RuntimeCapability.speech_alignment:
        if not model_path.is_dir():
            raise server_error("model_missing", "Forced alignment model directory is missing")
        expected = params.capability_spec_id.removesuffix("-v1") + f"-{device}-v1"
        if params.device_profile.execution_profile_id != expected:
            raise server_error("runtime_mismatch", "Forced alignment model and execution profile differ")
        return AlignmentEngine(model_path, device, threads, params.capability_spec_id)
    if capability is RuntimeCapability.speech_diarization:
        if device != "cpu" or not model_path.is_dir():
            raise server_error("unsupported_device", "Sherpa-ONNX diarization requires CPU")
        return DiarizationEngine(model_path, device, threads, params.capability_spec_id)
    if capability is RuntimeCapability.speech_synthesis:
        if device not in {"cpu", "mps", "cuda", "vulkan"} or not model_path.is_dir():
            raise server_error("unsupported_device", "Synthesis profile is unsupported")
        quantization = {
            FULL_MODEL_SPEC_ID: "orig",
            Q8_MODEL_SPEC_ID: "q8_0",
        }.get(params.capability_spec_id)
        if quantization is not None:
            expected = f"voxcpm2-{device}-gguf-{quantization}-v1"
            if params.device_profile.execution_profile_id != expected:
                raise server_error("runtime_mismatch", "VoxCPM2 model and execution profile differ")
        return SynthesisEngine(model_path, device, threads, params.capability_spec_id)
    if capability is RuntimeCapability.speech_separation:
        if device != "cpu" or not model_path.is_dir():
            raise server_error("unsupported_device", "Spleeter source separation requires CPU")
        from knovum_inference.speech.separation import SpleeterSourceSeparator

        return SpleeterSourceSeparator(model_path, work_directory, threads, params.capability_spec_id)
    if capability is RuntimeCapability.image_segmentation:
        if device != "cpu" or not model_path.is_file():
            raise server_error("unsupported_device", "Image segmentation profile is unsupported")
        from knovum_inference.image_segmentation import ImageSegmentationEngine

        return ImageSegmentationEngine(str(model_path), str(work_directory), threads)
    raise server_error("runtime_mismatch", "Runtime capability is not implemented")
