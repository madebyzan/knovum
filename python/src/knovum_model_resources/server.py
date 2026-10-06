from __future__ import annotations

import contextlib
import json
import os
import sys

from .contracts import DownloadRequest, HuggingFaceSource
from .progress import DownloadProgress

MAX_REQUEST_BYTES = 1024 * 1024


def main() -> None:
    os.environ["TQDM_DISABLE"] = "1"
    output = sys.stdout
    payload = sys.stdin.buffer.readline(MAX_REQUEST_BYTES + 1)
    try:
        if len(payload) > MAX_REQUEST_BYTES:
            raise ValueError("Model download request exceeds the message limit")
        request = DownloadRequest.model_validate_json(payload)
        progress = DownloadProgress(output, sum(file.size for file in request.files))
        with contextlib.redirect_stdout(sys.stderr):
            if isinstance(request.source, HuggingFaceSource):
                from .hugging_face import download as download_hugging_face

                download_hugging_face(request, request.source, progress)
            else:
                from .model_scope import download as download_model_scope

                download_model_scope(request, request.source, progress)
        progress.complete()
    except Exception as error:
        output.write(
            json.dumps({"event": "error", "message": f"Model download failed ({type(error).__name__})"}) + "\n"
        )
        output.flush()
        raise SystemExit(1) from None
