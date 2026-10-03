import asyncio

from knovum_inference.worker import Worker


def main() -> None:
    asyncio.run(Worker().serve())
