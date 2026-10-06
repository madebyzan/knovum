from __future__ import annotations

import os

from .contracts import DownloadRequest, HuggingFaceSource
from .progress import DownloadProgress


def download(request: DownloadRequest, source: HuggingFaceSource, progress: DownloadProgress) -> None:
    # A mirror is an explicitly selected HTTP endpoint, never an implicit Xet route.
    os.environ["HF_HUB_DISABLE_XET"] = "1" if source.endpoint == "https://hf-mirror.com" else "0"
    from huggingface_hub import snapshot_download
    from tqdm.auto import tqdm

    class ByteProgress(tqdm):  # type: ignore[misc]
        def __init__(self, *args: object, **kwargs: object) -> None:
            # Keep byte accounting active while display() suppresses terminal rendering.
            kwargs["disable"] = False
            super().__init__(*args, **kwargs)

        def update(self, n: float | None = 1) -> bool | None:
            changed: bool | None = super().update(n)
            if self.unit == "B":
                progress.update(int(self.n))
            return changed

        def display(self, msg: str | None = None, pos: int | None = None) -> None:
            return None

    snapshot_download(
        repo_id=source.repository,
        revision=source.revision,
        endpoint=source.endpoint,
        local_dir=request.directory,
        allow_patterns=[file.path for file in request.files],
        token=False,
        max_workers=2,
        tqdm_class=ByteProgress,
    )
