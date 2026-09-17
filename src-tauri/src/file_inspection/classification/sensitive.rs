use std::path::Path;

use super::file_name_lower;

const NAMES: &[&str] = &[
    ".aws-credentials",
    ".git-credentials",
    ".htpasswd",
    ".my.cnf",
    ".netrc",
    ".npmrc",
    ".pgpass",
    ".pypirc",
    ".yarnrc",
    "credentials",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "id_rsa",
];

const SUFFIXES: &[&str] = &[".kdbx", ".keystore", ".key", ".p12", ".pem", ".pfx"];

pub fn is_sensitive_file(path: &Path) -> bool {
    let Some(name) = file_name_lower(path) else {
        return false;
    };
    name == ".env"
        || name.starts_with(".env.")
        || NAMES.contains(&name.as_str())
        || SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
}
