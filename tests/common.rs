use std::{io, os::unix::fs::MetadataExt, path::Path, process::Command};

pub fn cargo<I, S>(dir: &Path, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let out = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    println!("{}", String::from_utf8(out.stderr).unwrap());
    println!("{}", String::from_utf8(out.stdout).unwrap());

    assert!(out.status.success());
}

pub fn ls_ascii(path: &Path, indent: usize) -> io::Result<String> {
    let mut entries = path.read_dir()?;
    let mut out = Vec::new();

    out.push(format!(
        "{}{}/",
        "  ".repeat(indent),
        path.file_name().unwrap().to_string_lossy()
    ));

    let indent = indent + 1;
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    while let Some(Ok(entry)) = entries.next() {
        let path = entry.path().to_path_buf();

        if entry.file_type()?.is_dir() {
            dirs.push(path);
        } else {
            files.push(path);
        }
    }

    dirs.sort();
    files.sort();

    for file in files {
        out.push(format!(
            "{}{} ({} bytes)",
            "  ".repeat(indent),
            file.file_name().unwrap().to_string_lossy(),
            file.metadata().unwrap().size()
        ));
    }

    for path in dirs {
        out.push(ls_ascii(&path, indent)?);
    }
    Ok(out.join("\n"))
}
