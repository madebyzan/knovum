import asyncio

from knovum_inference.runtime.worker import Worker


def main() -> None:
    asyncio.run(Worker().serve())
