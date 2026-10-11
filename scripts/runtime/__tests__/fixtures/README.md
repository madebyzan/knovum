ZIP fixtures exercise selected-entry extraction: regular bytes, symlinks, duplicate names and absent
entries. TAR fixtures exercise runtime preflight: 10,000 regular zero-byte entries whose verbose
metadata exceeds 1 MiB, path traversal, a symbolic link, and a duplicate name. None are extracted.

These immutable archives were generated once with Python's standard archive libraries as developer
fixtures. Runtime producers and these tests use Node and maintained archive readers or the OS tar
command; fixture generation adds no Python runtime dependency.
