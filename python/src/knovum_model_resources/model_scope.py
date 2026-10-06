from __future__ import annotations

from .contracts import DownloadRequest, ModelScopeSource
from .progress import DownloadProgress


def download(request: DownloadRequest, source: ModelScopeSource, progress: DownloadProgress) -> None:
    from modelscope_hub import HubApi, ProgressCallback

    class FileProgress(ProgressCallback):
        def __init__(self, filename: str, file_size: int) -> None:
            super().__init__(filename, file_size)
            progress.update_file(filename, 0)

        def update(self, size: int) -> None:
            progress.advance_file(self.filename, size)

        def end(self) -> None:
            progress.update_file(self.filename, self.file_size)

    api = HubApi(endpoint="https://modelscope.cn", token="")
    api.download_repo(
        repo_id=source.repository,
        repo_type="model",
        revision=source.revision,
        local_dir=request.directory,
        allow_patterns=[file.path for file in request.files],
        max_workers=2,
        progress_callbacks=[FileProgress],
    )
