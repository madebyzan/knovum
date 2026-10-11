fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("contract exporter must be built in the project checkout");
    if let Err(error) = knovum_lib::contract_codegen::export(&root) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
