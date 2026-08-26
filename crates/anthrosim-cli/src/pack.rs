use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, BufReader, Read, Seek, Write},
    path::{Component, Path, PathBuf},
};

use crate::bundle::{self, artifact_fs};

const ZIP_LOCAL_FILE_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_DIRECTORY_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_VERSION: u16 = 20;
const ZIP_STORED: u16 = 0;
const DOS_TIME_MIDNIGHT: u16 = 0;
const DOS_DATE_1980_01_01: u16 = (1 << 5) | 1;

const CRC32_TABLE: [u32; 256] = crc32_table();

#[derive(Debug)]
struct ArchiveEntry {
    name: String,
    crc32: u32,
    size: u32,
    local_header_offset: u32,
}

pub fn pack_completed_run(
    run_dir: &Path,
    output: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let canonical_run_dir = fs::canonicalize(run_dir)?;
    let files = bundle::validated_bundle_files(&canonical_run_dir)?;
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| run_dir.with_extension("zip"));
    let resolved_output = resolve_path_for_containment(&output)?;

    if resolved_output.starts_with(&canonical_run_dir) {
        return Err(invalid_input(format!(
            "archive output must be outside the source run directory: {}",
            output.display()
        ))
        .into());
    }
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let mut temp_os = output.as_os_str().to_os_string();
    temp_os.push(".tmp");
    let temp = PathBuf::from(temp_os);
    if temp.exists() {
        fs::remove_file(&temp)?;
    }

    if let Err(error) = write_zip(&temp, &canonical_run_dir, &files) {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }

    if output.exists() {
        fs::remove_file(&output)?;
    }
    fs::rename(&temp, &output)?;
    Ok(output)
}

fn resolve_path_for_containment(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let normalized = normalize_path(&absolute);
    if normalized.exists() {
        return fs::canonicalize(normalized);
    }

    let mut ancestor = normalized.as_path();
    let mut missing = Vec::<OsString>::new();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            invalid_input(format!(
                "archive output has no resolvable existing ancestor: {}",
                path.display()
            ))
        })?;
        missing.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            invalid_input(format!(
                "archive output has no resolvable existing ancestor: {}",
                path.display()
            ))
        })?;
    }

    let mut resolved = fs::canonicalize(ancestor)?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::Normal(value) => normalized.push(value),
        }
    }
    normalized
}

fn write_zip(path: &Path, run_dir: &Path, files: &[(String, PathBuf)]) -> io::Result<()> {
    if files.len() > usize::from(u16::MAX) {
        return Err(invalid_input("too many files for classic ZIP archive"));
    }

    let mut output = File::create(path)?;
    let mut entries = Vec::with_capacity(files.len());

    for (name, source_path) in files {
        let source_path = artifact_fs::canonical_regular_file_within(
            run_dir,
            source_path,
            "bundle artifact selected for packing",
        )?;
        let name_bytes = name.as_bytes();
        let name_len = u16::try_from(name_bytes.len())
            .map_err(|_| invalid_input("archive filename is too long for classic ZIP"))?;
        let (crc32, size) = file_crc32_and_size(&source_path)?;
        let local_header_offset = u32::try_from(output.stream_position()?)
            .map_err(|_| invalid_input("archive exceeds classic ZIP 4 GiB offset limit"))?;

        write_u32(&mut output, ZIP_LOCAL_FILE_HEADER)?;
        write_u16(&mut output, ZIP_VERSION)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, ZIP_STORED)?;
        write_u16(&mut output, DOS_TIME_MIDNIGHT)?;
        write_u16(&mut output, DOS_DATE_1980_01_01)?;
        write_u32(&mut output, crc32)?;
        write_u32(&mut output, size)?;
        write_u32(&mut output, size)?;
        write_u16(&mut output, name_len)?;
        write_u16(&mut output, 0)?;
        output.write_all(name_bytes)?;

        let mut source = BufReader::new(File::open(&source_path)?);
        let copied = io::copy(&mut source, &mut output)?;
        if copied != u64::from(size) {
            return Err(invalid_input(format!(
                "artifact changed while packing: {}",
                source_path.display()
            )));
        }

        entries.push(ArchiveEntry {
            name: name.clone(),
            crc32,
            size,
            local_header_offset,
        });
    }

    let central_offset = u32::try_from(output.stream_position()?)
        .map_err(|_| invalid_input("archive exceeds classic ZIP 4 GiB offset limit"))?;

    for entry in &entries {
        let name_bytes = entry.name.as_bytes();
        let name_len = u16::try_from(name_bytes.len())
            .map_err(|_| invalid_input("archive filename is too long for classic ZIP"))?;

        write_u32(&mut output, ZIP_CENTRAL_DIRECTORY_HEADER)?;
        write_u16(&mut output, ZIP_VERSION)?;
        write_u16(&mut output, ZIP_VERSION)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, ZIP_STORED)?;
        write_u16(&mut output, DOS_TIME_MIDNIGHT)?;
        write_u16(&mut output, DOS_DATE_1980_01_01)?;
        write_u32(&mut output, entry.crc32)?;
        write_u32(&mut output, entry.size)?;
        write_u32(&mut output, entry.size)?;
        write_u16(&mut output, name_len)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u32(&mut output, 0)?;
        write_u32(&mut output, entry.local_header_offset)?;
        output.write_all(name_bytes)?;
    }

    let central_end = u32::try_from(output.stream_position()?)
        .map_err(|_| invalid_input("archive exceeds classic ZIP 4 GiB offset limit"))?;
    let central_size = central_end
        .checked_sub(central_offset)
        .ok_or_else(|| invalid_input("invalid ZIP central directory offsets"))?;
    let entry_count = u16::try_from(entries.len())
        .map_err(|_| invalid_input("too many files for classic ZIP archive"))?;

    write_u32(&mut output, ZIP_END_OF_CENTRAL_DIRECTORY)?;
    write_u16(&mut output, 0)?;
    write_u16(&mut output, 0)?;
    write_u16(&mut output, entry_count)?;
    write_u16(&mut output, entry_count)?;
    write_u32(&mut output, central_size)?;
    write_u32(&mut output, central_offset)?;
    write_u16(&mut output, 0)?;
    output.flush()?;
    Ok(())
}

fn file_crc32_and_size(path: &Path) -> io::Result<(u32, u32)> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(invalid_input(format!(
            "artifact selected for packing is not a non-symlink regular file: {}",
            path.display()
        )));
    }
    let size = u32::try_from(metadata.len()).map_err(|_| {
        invalid_input(format!(
            "artifact is too large for classic ZIP (>4 GiB): {}",
            path.display()
        ))
    })?;

    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = [0_u8; 64 * 1024];
    let mut crc = 0xffff_ffff_u32;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            let index = ((crc ^ u32::from(*byte)) & 0xff) as usize;
            crc = CRC32_TABLE[index] ^ (crc >> 8);
        }
    }
    Ok((!crc, size))
}

fn write_u16(output: &mut File, value: u16) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

fn write_u32(output: &mut File, value: u32) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

const fn crc32_table() -> [u32; 256] {
    let mut table = [0_u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut crc = index as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[index] = crc;
        index += 1;
    }
    table
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        sync::atomic::{AtomicU64, Ordering},
    };

    use anthrosim_core::{ExperimentConfig, PopulationConfig, Simulation, WorldConfig};

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn pack_is_deterministic_and_excludes_unrelated_files() {
        let root = test_dir("deterministic");
        write_real_completed_bundle(&root);
        fs::write(root.join("notes.txt"), "do not share\n").unwrap();

        let first = root.with_extension("first.zip");
        let second = root.with_extension("second.zip");
        pack_completed_run(&root, Some(&first)).unwrap();
        pack_completed_run(&root, Some(&second)).unwrap();

        assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
        let names = central_directory_names(&fs::read(&first).unwrap());
        assert_eq!(
            names,
            BTreeSet::from([
                "checkpoint.json".to_string(),
                "events.json".to_string(),
                "initial-population.json".to_string(),
                "manifest.json".to_string(),
                "metrics.json".to_string(),
                "world.json".to_string(),
            ])
        );
        assert!(!names.contains("notes.txt"));
        cleanup(&root, &[&first, &second]);
    }

    #[cfg(unix)]
    #[test]
    fn pack_rejects_symlinked_canonical_artifact() {
        use std::os::unix::fs::symlink;

        let root = test_dir("symlink-source");
        write_real_completed_bundle(&root);
        let outside = root.with_extension("outside-world.json");
        fs::rename(root.join("world.json"), &outside).unwrap();
        symlink(&outside, root.join("world.json")).unwrap();
        let output = root.with_extension("symlink.zip");

        let error = pack_completed_run(&root, Some(&output))
            .unwrap_err()
            .to_string();
        assert!(error.contains("symbolic link"));
        assert!(!output.exists());

        cleanup(&root, &[&output, &outside]);
    }

    #[test]
    fn output_inside_run_directory_is_rejected_without_modifying_bundle() {
        let root = test_dir("inside-run-dir");
        write_real_completed_bundle(&root);
        let before = snapshot_bundle(&root);
        let output = root.join("exports").join("run.zip");

        let error = pack_completed_run(&root, Some(&output))
            .unwrap_err()
            .to_string();

        assert!(error.contains("outside the source run directory"));
        assert_eq!(snapshot_bundle(&root), before);
        assert!(!output.exists());
        assert!(!root.join("exports").exists());
        cleanup(&root, &[]);
    }

    #[test]
    fn canonical_artifact_output_alias_is_rejected_without_modifying_bundle() {
        let root = test_dir("artifact-alias");
        write_real_completed_bundle(&root);
        let before = snapshot_bundle(&root);
        let output = root.join("nested").join("..").join("manifest.json");

        let error = pack_completed_run(&root, Some(&output))
            .unwrap_err()
            .to_string();

        assert!(error.contains("outside the source run directory"));
        assert_eq!(snapshot_bundle(&root), before);
        cleanup(&root, &[]);
    }

    #[test]
    fn paused_bundle_without_manifest_is_rejected() {
        let root = test_dir("paused");
        write_real_completed_bundle(&root);
        fs::remove_file(root.join("manifest.json")).unwrap();

        let error = pack_completed_run(&root, None).unwrap_err().to_string();
        assert!(error.contains("manifest.json"));
        cleanup(&root, &[]);
    }

    #[test]
    fn malformed_json_is_rejected() {
        let root = test_dir("invalid-json");
        write_real_completed_bundle(&root);
        fs::write(root.join("metrics.json"), "{").unwrap();

        let error = pack_completed_run(&root, None).unwrap_err().to_string();
        assert!(error.contains("metrics.json"));
        cleanup(&root, &[]);
    }

    fn write_real_completed_bundle(root: &Path) {
        let config = ExperimentConfig::new(71, 0)
            .with_world(WorldConfig::new(4, 4))
            .with_population(
                PopulationConfig::new(8)
                    .with_target_household_size(2)
                    .with_max_person_records(64),
            );
        let simulation = Simulation::new(config).unwrap();
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();

        fs::create_dir_all(root).unwrap();
        write_json(&root.join("world.json"), &world);
        write_json(&root.join("initial-population.json"), &initial_population);
        write_json(&root.join("manifest.json"), &recorded.manifest);
        write_json(&root.join("events.json"), recorded.events());
        write_json(&root.join("metrics.json"), recorded.metrics());
        write_json(&root.join("checkpoint.json"), &recorded.checkpoint);
    }

    fn write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) {
        let json = serde_json::to_string_pretty(value).unwrap();
        fs::write(path, format!("{json}\n")).unwrap();
    }

    fn snapshot_bundle(root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut files = fs::read_dir(root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let bytes = fs::read(entry.path()).unwrap();
                (name, bytes)
            })
            .collect::<Vec<_>>();
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files
    }

    fn central_directory_names(bytes: &[u8]) -> BTreeSet<String> {
        let signature = ZIP_CENTRAL_DIRECTORY_HEADER.to_le_bytes();
        let mut names = BTreeSet::new();
        let mut offset = 0;
        while offset + 46 <= bytes.len() {
            if bytes[offset..offset + 4] != signature {
                offset += 1;
                continue;
            }
            let name_len = u16::from_le_bytes([bytes[offset + 28], bytes[offset + 29]]) as usize;
            let extra_len = u16::from_le_bytes([bytes[offset + 30], bytes[offset + 31]]) as usize;
            let comment_len = u16::from_le_bytes([bytes[offset + 32], bytes[offset + 33]]) as usize;
            let name_start = offset + 46;
            let name_end = name_start + name_len;
            assert!(name_end <= bytes.len());
            names.insert(String::from_utf8(bytes[name_start..name_end].to_vec()).unwrap());
            offset = name_end + extra_len + comment_len;
        }
        names
    }

    fn test_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-pack-{label}-{}-{id}",
            std::process::id()
        ))
    }

    fn cleanup(root: &Path, files: &[&Path]) {
        let _ = fs::remove_dir_all(root);
        for file in files {
            let _ = fs::remove_file(file);
        }
        let default_archive = root.with_extension("zip");
        let _ = fs::remove_file(default_archive);
    }
}
