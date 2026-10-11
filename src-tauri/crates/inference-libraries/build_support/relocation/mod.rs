use std::{fs, io, path::Path, process::Command};
fn output(program: &str, args: &[&str], path: &Path) -> io::Result<String> {
    let process_output = Command::new(program).args(args).arg(path).output()?;
    if !process_output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&process_output.stderr).to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&process_output.stdout).into_owned())
}
fn run(args: &[&str], path: &Path) -> io::Result<()> {
    output("install_name_tool", args, path).map(|_| ())
}
pub fn materialize(root: &Path, at: &Path) -> io::Result<()> {
    for entry in fs::read_dir(at)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            materialize(root, &entry.path())?;
        } else if entry.file_type()?.is_symlink() {
            let target = entry.path().canonicalize()?;
            if !target.starts_with(root.canonicalize()?) {
                return Err(io::Error::other("Compiled alias escapes install tree"));
            }
            let bytes = fs::read(target)?;
            fs::remove_file(entry.path())?;
            fs::write(entry.path(), bytes)?;
        }
    }
    Ok(())
}
pub fn relocate(root: &Path, files: &[String]) -> io::Result<()> {
    let libraries: Vec<_> = files
        .iter()
        .filter(|p| p.ends_with(".dylib") || p.ends_with(".so"))
        .collect();
    for file in &libraries {
        // The qualified ONNX dylib has only system dependencies.
        if file.starts_with("libonnxruntime.") {
            continue;
        }
        let path = root.join(file);
        let id = output("otool", &["-D"], &path)?
            .lines()
            .nth(1)
            .map(|p| p.trim().to_owned());
        if id.is_some() {
            run(
                &[
                    "-id",
                    &format!(
                        "@loader_path/{}",
                        Path::new(file).file_name().unwrap().to_str().unwrap()
                    ),
                ],
                &path,
            )?;
        }
        let dependencies = output("otool", &["-L"], &path)?;
        for line in dependencies.lines().skip(1) {
            let name = line.trim().split(" (").next().unwrap();
            if Some(name) == id.as_deref()
                || name.starts_with("/usr/lib/")
                || name.starts_with("/System/Library/")
            {
                continue;
            }
            let owner = dependency_owner(file, name, &libraries)?;
            let from: Vec<_> = Path::new(file).parent().unwrap().components().collect();
            let to: Vec<_> = Path::new(owner).components().collect();
            let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
            let relative = std::iter::repeat_n("..".to_owned(), from.len() - common)
                .chain(
                    to.iter()
                        .skip(common)
                        .map(|p| p.as_os_str().to_str().unwrap().to_owned()),
                )
                .collect::<Vec<_>>()
                .join("/");
            run(
                &["-change", name, &format!("@loader_path/{relative}")],
                &path,
            )?;
        }
        let commands = output("otool", &["-l"], &path)?;
        let mut rpath = false;
        for line in commands.lines().map(str::trim) {
            if line == "cmd LC_RPATH" {
                rpath = true;
            }
            if rpath && line.starts_with("path ") {
                let name = line
                    .trim_start_matches("path ")
                    .split(" (offset")
                    .next()
                    .unwrap();
                if !name.starts_with("@loader_path") {
                    run(&["-delete_rpath", name], &path)?;
                }
                rpath = false;
            }
        }
    }
    Ok(())
}
fn dependency_owner<'a>(file: &str, name: &str, libraries: &[&'a String]) -> io::Result<&'a str> {
    let candidates: Vec<_> = libraries
        .iter()
        .filter(|p| Path::new(p).file_name() == Path::new(name).file_name())
        .collect();
    let same: Vec<_> = candidates
        .iter()
        .filter(|p| Path::new(p.as_str()).parent() == Path::new(file).parent())
        .collect();
    let owned: Vec<_> = candidates
        .iter()
        .filter(|p| {
            p.starts_with(if file.starts_with("compiled-libraries/whisper/") {
                "compiled-libraries/whisper/"
            } else {
                "bin/"
            })
        })
        .collect();
    if same.len() == 1 {
        Ok(same[0].as_str())
    } else if owned.len() == 1 {
        Ok(owned[0].as_str())
    } else if candidates.len() == 1 {
        Ok(candidates[0].as_str())
    } else {
        Err(io::Error::other(format!(
            "Compiled dependency has no unique packaged owner: {file}: {name}"
        )))
    }
}

#[cfg(test)]
mod tests;
