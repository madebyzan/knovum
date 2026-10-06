"""Start the application-owned Agent Runtime process with native certificate trust."""


def main() -> None:
    import truststore

    truststore.inject_into_ssl()

    from knovum_agent_runtime.server import main as serve

    serve()


main()
