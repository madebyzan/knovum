/// The managed Conda runtime keeps shared Windows libraries in Library/bin.
/// Keep the DLL-directory handle alive for the whole isolated worker process.
pub(crate) fn isolated_bootstrap(entry: &str) -> String {
    format!(
        "import os,sys; _knovum_native_dll = os.add_dll_directory(os.path.join(sys.prefix,'Library','bin')) if os.name == 'nt' else None; sys.path[:0]=[sys.argv[1],sys.argv[2]]; {entry}"
    )
}
