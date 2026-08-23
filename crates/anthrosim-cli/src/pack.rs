use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
};

use serde::Deserialize;

const REQUIRED_JSON: &[&str] = &[
    "checkpoint.json",
    "events.json",
    "manifest.json",
    "metrics.json",
    "world.json",
];

const POPULATION_JSON: &[&str] = &["initial-population.json", "resume-start-population.json"];

const OPTIONAL_JSON: &[&str] = &[
    "completion.json",
    "evidence.json",
    "landscape-checkpoint.json",
    "landscape-manifest.json",
    "landscape.json",
    "spatial-mechanisms.json",
    "spatial-observability.json",
];

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
    let files = validated_bundle_files(run_dir)?;
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| run_dir.with_extension("zip"));

    if output == run_dir {
        return Err(invalid_input("archive output cannot be the run directory").into());
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

    if let Err(error) = write_zip(&temp, &files) {
        let _ = fs::remove_file(&temp);
        return Err(error.into());
    }

    if output.exists() {
        fs::remove_file(&output)?;
    }
    fs::rename(&temp, &output)?;
    Ok(output)
}

fn validated_bundle_files(run_dir: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    if !run_dir.is_dir() {
        return Err(invalid_input(format!(
            "run directory does not exist or is not a directory: {}",
            run_dir.display()
        )));
    }

    let mut names = Vec::new();
    for name in REQUIRED_JSON {
        require_json(run_dir, name, &mut names)?;
    }

    let mut has_population = false;
    for name in POPULATION_JSON {
        let path = run_dir.join(name);
        if path.is_file() {
            validate_json(&path)?;
            names.push((*name).to_string());
            has_population = true;
        } else if path.exists() {
            return Err(invalid_input(format!(
                "expected bundle artifact is not a regular file: {}",
                path.display()
            )));
        }
    }
    if !has_population {
        return Err(invalid_input(
            "completed run bundle must contain initial-population.json or resume-start-population.json",
        ));
    }

    for name in OPTIONAL_JSON {
        let path = run_dir.join(name);
        if path.is_file() {
            validate_json(&path)?;
            names.push((*name).to_string());
        } else if path.exists() {
            return Err(invalid_input(format!(
                "expected bundle artifact is not a regular file: {}",
                path.display()
            )));
        }
    }

    let has_landscape = names.iter().any(|name| name == "landscape.json");
    let has_landscape_manifest = names.iter().any(|name| name == "landscape-manifest.json");
    let has_landscape_checkpoint = names.iter().any(|name| name == "landscape-checkpoint.json");
    let has_spatial_mechanisms = names.iter().any(|name| name == "spatial-mechanisms.json");
    let has_spatial_observability = names
        .iter()
        .any(|name| name == "spatial-observability.json");

    if has_landscape && (!has_landscape_manifest || !has_landscape_checkpoint) {
        return Err(invalid_input(
            "completed landscape-bound run must contain landscape-manifest.json and landscape-checkpoint.json",
        ));
    }
    if !has_landscape
        && (has_landscape_manifest
            || has_landscape_checkpoint
            || has_spatial_mechanisms
            || has_spatial_observability)
    {
        return Err(invalid_input(
            "landscape/spatial artifacts require landscape.json in the same completed run bundle",
        ));
    }

    names.sort_unstable();
    names.dedup();
    Ok(names
        .into_iter()
        .map(|name| {
            let path = run_dir.join(&name);
            (name, path)
        })
        .collect())
}

fn require_json(run_dir: &Path, name: &str, names: &mut Vec<String>) -> io::Result<()> {
    let path = run_dir.join(name);
    if !path.is_file() {
        return Err(invalid_input(format!(
            "completed run bundle is missing required artifact: {}",
            path.display()
        )));
    }
    validate_json(&path)?;
    names.push(name.to_string());
    Ok(())
}

fn validate_json(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    let mut deserializer = serde_json::Deserializer::from_reader(BufReader::new(file));
    serde::de::IgnoredAny::deserialize(&mut deserializer)
        .map_err(|error| invalid_input(format!("invalid JSON in {}: {error}", path.display())))?;
    deserializer.end().map_err(|error| {
        invalid_input(format!(
            "invalid trailing JSON in {}: {error}",
            path.display()
        ))
    })?;
    Ok(())
}

fn write_zip(path: &Path, files: &[(String, PathBuf)]) -> io::Result<()> {
    if files.len() > usize::from(u16::MAX) {
        return Err(invalid_input("too many files for classic ZIP archive"));
    }

    let mut output = File::create(path)?;
    let mut entries = Vec::with_capacity(files.len());

    for (name, source_path) in files {
        let name_bytes = name.as_bytes();
        let name_len = u16::try_from(name_bytes.len())
            .map_err(|_| invalid_input("archive filename is too long for classic ZIP"))?;
        let (crc32, size) = file_crc32_and_size(source_path)?;
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

        let mut source = BufReader::new(File::open(source_path)?);
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
    let metadata = fs::metadata(path)?;
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

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn pack_is_deterministic_and_excludes_unrelated_files() {
        let root = test_dir("deterministic");
        write_minimal_completed_bundle(&root);
        fs::write(root.join("completion.json"), "{}\n").unwrap();
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
                "completion.json".to_string(),
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

    #[test]
    fn paused_bundle_without_manifest_is_rejected() {
        let root = test_dir("paused");
        write_minimal_completed_bundle(&root);
        fs::remove_file(root.join("manifest.json")).unwrap();

        let error = pack_completed_run(&root, None).unwrap_err().to_string();
        assert!(error.contains("manifest.json"));
        cleanup(&root, &[]);
    }

    #[test]
    fn malformed_json_is_rejected() {
        let root = test_dir("invalid-json");
        write_minimal_completed_bundle(&root);
        fs::write(root.join("metrics.json"), "{").unwrap();

        let error = pack_completed_run(&root, None).unwrap_err().to_string();
        assert!(error.contains("invalid JSON"));
        cleanup(&root, &[]);
    }

    fn write_minimal_completed_bundle(root: &Path) {
        fs::create_dir_all(root).unwrap();
        for name in REQUIRED_JSON {
            fs::write(root.join(name), "{}\n").unwrap();
        }
        fs::write(root.join("initial-population.json"), "{}\n").unwrap();
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
