"""Start the application-owned Assistant process with native certificate trust."""


def main() -> None:
    import truststore

    truststore.inject_into_ssl()

    from knovum_assistant.server import main as serve

    serve()


main()
