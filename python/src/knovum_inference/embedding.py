from __future__ import annotations

import math
import os
from pathlib import Path
from typing import Any, Literal

from knovum_inference.errors import server_error

EXPECTED_DIMENSIONS = 1024
MAX_TOKENS = 2048
QUERY_PROMPT = (
    "Instruct: Given a knowledge-space search query, retrieve relevant passages from the "
    "user's documents, notes, media transcripts, code, and web captures.\nQuery:"
)
REQUIRED_MODEL_FILES = (
    "config.json",
    "config_sentence_transformers.json",
    "model.safetensors",
    "modules.json",
    "tokenizer.json",
    "tokenizer_config.json",
    "1_Pooling/config.json",
)


class EmbeddingEngine:
    def __init__(self, model_directory: str, device: str, cpu_threads: int) -> None:
        self._model_directory = Path(model_directory)
        self._device = device
        self._cpu_threads = cpu_threads
        self._model: Any | None = None

    def encode(self, texts: list[str], role: Literal["query", "document"]) -> tuple[list[list[float]], int]:
        model = self._load()
        prompt = QUERY_PROMPT if role == "query" else ""
        prepared = [f"{prompt}{text}" for text in texts]
        tokenized = model.tokenizer(
            prepared,
            add_special_tokens=True,
            padding=False,
            truncation=False,
            return_attention_mask=False,
            return_token_type_ids=False,
        )["input_ids"]
        counts = [len(token_ids) for token_ids in tokenized]
        if any(count > MAX_TOKENS for count in counts):
            raise server_error("invalid_output", "Embedding input exceeds 2048 tokens")
        kwargs = {"prompt": QUERY_PROMPT} if role == "query" else {}
        vectors = model.encode(
            texts,
            convert_to_numpy=True,
            normalize_embeddings=True,
            show_progress_bar=False,
            **kwargs,
        )
        ordered = vectors.astype("float32", copy=False).tolist()
        if len(ordered) != len(texts) or any(
            len(vector) != EXPECTED_DIMENSIONS or any(not math.isfinite(value) for value in vector)
            for vector in ordered
        ):
            raise server_error("invalid_output", "Embedding model returned invalid dimensions")
        return ordered, sum(counts)

    def _load(self) -> Any:
        if self._model is not None:
            return self._model
        if self._device != "cpu":
            raise server_error("unsupported_device", f"Unsupported device: {self._device}")
        if not self._model_directory.is_dir():
            raise server_error("model_missing", "Embedding model directory is missing")
        missing = [relative for relative in REQUIRED_MODEL_FILES if not (self._model_directory / relative).is_file()]
        if missing:
            raise server_error(
                "model_corrupt",
                f"Embedding model snapshot is incomplete: {', '.join(missing)}",
            )
        os.environ["HF_HUB_OFFLINE"] = "1"
        os.environ["HF_HUB_DISABLE_TELEMETRY"] = "1"
        os.environ["TRANSFORMERS_OFFLINE"] = "1"
        os.environ["TOKENIZERS_PARALLELISM"] = "false"
        os.environ["OMP_NUM_THREADS"] = str(self._cpu_threads)
        os.environ["MKL_NUM_THREADS"] = str(self._cpu_threads)
        try:
            import torch
            from sentence_transformers import SentenceTransformer, models

            torch.set_num_threads(self._cpu_threads)
            torch.set_num_interop_threads(1)
            model = SentenceTransformer(
                str(self._model_directory),
                device=self._device,
                local_files_only=True,
                trust_remote_code=False,
            )
        except Exception as error:
            raise server_error("model_corrupt", "Embedding model failed to load") from error
        if model.get_sentence_embedding_dimension() != EXPECTED_DIMENSIONS:
            raise server_error("model_corrupt", "Embedding model has unexpected dimensions")
        pooling = [module for module in model.modules() if isinstance(module, models.Pooling)]
        if len(pooling) != 1 or not _uses_only_last_token_pooling(pooling[0]):
            raise server_error("model_corrupt", "Embedding model must use last-token pooling")
        model.max_seq_length = MAX_TOKENS
        self._model = model
        return model


def _uses_only_last_token_pooling(pooling: Any) -> bool:
    return bool(
        pooling.pooling_mode_lasttoken
        and not pooling.pooling_mode_cls_token
        and not pooling.pooling_mode_max_tokens
        and not pooling.pooling_mode_mean_tokens
        and not pooling.pooling_mode_mean_sqrt_len_tokens
        and not pooling.pooling_mode_weightedmean_tokens
    )
