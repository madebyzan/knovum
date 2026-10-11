"""Start the application-owned Agent Runtime process with operating-system certificate trust."""


def main() -> None:
    import truststore

    truststore.inject_into_ssl()

    from knovum_agent_runtime.server import main as serve

    serve()


if __name__ == "__main__":
    main()
