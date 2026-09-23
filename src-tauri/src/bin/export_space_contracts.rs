fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must have a project root");
    if let Err(error) = knovum_lib::contract_codegen::export(root) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
