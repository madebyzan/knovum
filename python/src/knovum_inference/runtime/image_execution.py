from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TYPE_CHECKING, Any, TypeAlias, cast

from knovum_inference.contracts.generated.control import RuntimeCapability, RuntimeInitializeParams
from knovum_inference.contracts.generated.image import ImageComputeParams, TensorDataType, TensorFileReference
from knovum_inference.errors import invalid_params, server_error
from knovum_inference.runtime.active_request import ActiveRequest
from knovum_inference.runtime.session import Session
from knovum_inference.transport.protocol import ProtocolWriter

if TYPE_CHECKING:
    import numpy as np
    from numpy.typing import NDArray

    from knovum_inference.image_depth import ImageDepthEngine
    from knovum_inference.image_inpainting import ImageInpaintingEngine
    from knovum_inference.image_selection import ImageSelectionEngine
    from knovum_inference.image_upscaling import ImageUpscalingEngine

    ImageEngine: TypeAlias = ImageUpscalingEngine | ImageInpaintingEngine | ImageSelectionEngine | ImageDepthEngine
IMAGE_CAPABILITIES = frozenset(
    {
        RuntimeCapability.image_upscaling,
        RuntimeCapability.image_inpainting,
        RuntimeCapability.image_selection,
        RuntimeCapability.image_depth,
    }
)


def create_engine(params: RuntimeInitializeParams, directory: Path) -> ImageEngine:
    import onnxruntime as ort

    from knovum_inference.image_depth import ImageDepthEngine
    from knovum_inference.image_inpainting import ImageInpaintingEngine
    from knovum_inference.image_selection import ImageSelectionEngine
    from knovum_inference.image_upscaling import ImageUpscalingEngine

    model = params.model_paths.image_processing
    if model is None or params.device_profile.device != "cpu" or not directory.is_dir():
        raise server_error("unsupported_device", "Image processing requires a verified CPU snapshot")
    variant = model.variant.value
    allowed = {
        "realEsrganGeneralX4v3": RuntimeCapability.image_upscaling,
        "realEsrganX4plus": RuntimeCapability.image_upscaling,
        "migan": RuntimeCapability.image_inpainting,
        "efficientSamTi": RuntimeCapability.image_selection,
        "efficientSamS": RuntimeCapability.image_selection,
        "depthAnythingV2SmallUint8": RuntimeCapability.image_depth,
        "depthAnythingV2Small": RuntimeCapability.image_depth,
    }
    if allowed[variant] is not params.capability:
        raise server_error("runtime_mismatch", "Image model does not implement the requested capability")
    options = ort.SessionOptions()
    options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
    options.intra_op_num_threads = params.device_profile.cpu_threads
    options.inter_op_num_threads = 1
    if params.capability is RuntimeCapability.image_upscaling:
        return ImageUpscalingEngine(directory, variant, options)
    if params.capability is RuntimeCapability.image_inpainting:
        return ImageInpaintingEngine(directory, options)
    if params.capability is RuntimeCapability.image_selection:
        return ImageSelectionEngine(directory, variant, options)
    return ImageDepthEngine(directory, variant, options)


async def execute(
    params: ImageComputeParams, session: Session, active: ActiveRequest, writer: ProtocolWriter
) -> dict[str, Any]:
    tensor = await asyncio.to_thread(_compute, params, session, active)
    return {
        "requestId": params.request_id,
        "attemptEpoch": params.attempt_epoch,
        "modelSpecId": params.model_spec_id,
        "tensorRef": tensor.model_dump(by_alias=True, mode="json"),
    }


def _compute(params: ImageComputeParams, session: Session, active: ActiveRequest) -> TensorFileReference:
    import numpy as np

    from knovum_inference.image_depth import ImageDepthEngine
    from knovum_inference.image_inpainting import ImageInpaintingEngine
    from knovum_inference.image_selection import ImageSelectionEngine
    from knovum_inference.image_tensors import read_tensor, write_tensor
    from knovum_inference.image_upscaling import ImageUpscalingEngine

    tensor = read_tensor(session.work_directory, params.tensor_ref, 64 * 1024 * 1024)
    operation = params.computation.root
    if tensor.shape[:2] != (1, 3):
        raise invalid_params("Image tensor requires one RGB image")
    expected_dtype = TensorDataType.float32_little_endian if operation.kind == "depth" else TensorDataType.uint8
    if params.tensor_ref.data_type is not expected_dtype:
        raise invalid_params("Image tensor data type does not match the operation")
    pixels: NDArray[np.uint8] | NDArray[np.float32]
    if operation.kind == "upscale" and session.capability is RuntimeCapability.image_upscaling:
        pixels = cast(ImageUpscalingEngine, session.engine).upscale(tensor.astype(np.uint8), active.cancelled)
    elif operation.kind == "inpaint" and session.capability is RuntimeCapability.image_inpainting:
        if operation.mask_ref.data_type is not TensorDataType.uint8:
            raise invalid_params("Inpainting mask must be uint8")
        mask = read_tensor(session.work_directory, operation.mask_ref, 16 * 1024 * 1024).astype(np.uint8)
        pixels = cast(ImageInpaintingEngine, session.engine).inpaint(tensor.astype(np.uint8), mask, active.cancelled)
    elif operation.kind == "select" and session.capability is RuntimeCapability.image_selection:
        identity = f"{params.input_revision}:{params.tensor_ref.sha256}"
        pixels = cast(ImageSelectionEngine, session.engine).select(
            tensor.astype(np.uint8), identity, operation.points, operation.box_rect, active.cancelled
        )
    elif operation.kind == "depth" and session.capability is RuntimeCapability.image_depth:
        pixels = cast(ImageDepthEngine, session.engine).estimate(tensor.astype(np.float32), active.cancelled)
    else:
        raise server_error("runtime_mismatch", "Image operation differs from the frozen capability")
    if active.cancelled.is_set():
        raise server_error("cancelled", "Image request was cancelled")
    return write_tensor(session.work_directory, pixels, params.request_id)
