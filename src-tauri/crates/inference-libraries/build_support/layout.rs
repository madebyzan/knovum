use std::{fs, io, path::Path};
pub fn inventory(root: &Path) -> io::Result<Vec<String>> {
    fn visit(root: &Path, at: &Path, files: &mut Vec<String>) -> io::Result<()> {
        for entry in fs::read_dir(at)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(root, &entry.path(), files)?;
            } else if kind.is_file() {
                files.push(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .ok_or_else(|| io::Error::other("Non UTF-8 compiled path"))?
                        .replace('\\', "/"),
                );
            } else {
                return Err(io::Error::other(
                    "Compiled inventory contains a link or special file",
                ));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    files.sort();
    if files.is_empty() || files.len() > 256 {
        return Err(io::Error::other("Compiled inventory exceeds its bound"));
    }
    Ok(files)
}
pub fn require_libraries(files: &[String], windows: bool) -> io::Result<()> {
    let extension = if windows { "dll" } else { "dylib" };
    let mut required = vec![if windows {
        "onnxruntime.dll".to_owned()
    } else {
        "libonnxruntime.1.22.1.dylib".to_owned()
    }];
    for name in ["audiocpp", "knovum_voice", "sherpa-onnx-c-api"] {
        required.push(format!("compiled-libraries/{name}.{extension}"));
    }
    required.push(format!(
        "compiled-libraries/whisper/{}whisper.{extension}",
        if windows { "" } else { "lib" }
    ));
    if required.iter().any(|name| !files.contains(name)) {
        return Err(io::Error::other("Compiled runtime is incomplete"));
    }
    Ok(())
}
