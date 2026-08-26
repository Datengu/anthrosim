use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

// This shared source file is compiled independently into several CLI binaries; not every binary
// uses every boundary helper even though the helpers are used across the CLI as a whole.
#[allow(dead_code)]
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) fn regular_file_exists(path: &Path, role: &str) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{role} may not be a symbolic link: {}", path.display()),
        )),
        Ok(metadata) if !metadata.is_file() => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{role} is not a regular file: {}", path.display()),
        )),
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(crate) fn require_regular_file(path: &Path, role: &str) -> io::Result<()> {
    if regular_file_exists(path, role)? {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{role} is missing: {}", path.display()),
        ))
    }
}

pub(crate) fn read_to_string(path: &Path, role: &str) -> io::Result<String> {
    require_regular_file(path, role)?;
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;
    Ok(content)
}

#[allow(dead_code)]
pub(crate) fn canonical_regular_file_within(
    root: &Path,
    path: &Path,
    role: &str,
) -> io::Result<PathBuf> {
    require_regular_file(path, role)?;
    let canonical_root = fs::canonicalize(root)?;
    let canonical_path = fs::canonicalize(path)?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{role} resolves outside declared root {}: {}",
                canonical_root.display(),
                path.display()
            ),
        ));
    }
    Ok(canonical_path)
}

#[allow(dead_code)]
pub(crate) fn atomic_write(path: &Path, payload: &[u8], role: &str) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let destination_existed = regular_file_exists(path, role)?;
    let (temp, mut file) = create_unique_temp(path)?;
    let result = (|| {
        file.write_all(payload)?;
        file.sync_all()?;
        drop(file);

        // Re-check immediately before replacement so an attacker cannot leave a symlink or
        // special file at the derived-artifact path between the initial check and publication.
        let destination_still_exists = regular_file_exists(path, role)?;
        if destination_still_exists && !destination_existed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{role} changed while preparing replacement: {}",
                    path.display()
                ),
            ));
        }

        match fs::rename(&temp, path) {
            Ok(()) => Ok(()),
            Err(rename_error) if destination_still_exists => {
                // Windows does not replace an existing regular file with rename. Removing the
                // verified regular directory entry is safe: remove_file unlinks a symlink rather
                // than following it, and the second regular-file check above rejects one first.
                require_regular_file(path, role)?;
                fs::remove_file(path)?;
                fs::rename(&temp, path).map_err(|error| {
                    io::Error::new(
                        error.kind(),
                        format!(
                            "could not publish {role} after removing its verified previous file: {error}; initial rename error: {rename_error}"
                        ),
                    )
                })
            }
            Err(error) => Err(error),
        }
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[allow(dead_code)]
fn create_unique_temp(path: &Path) -> io::Result<(PathBuf, File)> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let base = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("anthrosim-artifact");

    loop {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{base}.anthrosim-tmp-{}-{id}", std::process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}
