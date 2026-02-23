use anyhow::Result;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

const QUICK_HASH_SIZE: usize = 64 * 1024; // 64KB

/// Compute a quick hash of the first 64KB of a file
pub fn compute_quick_hash(path: &Path) -> Result<String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; QUICK_HASH_SIZE];

    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let hash = xxh3_64(&buffer);
    Ok(format!("{:016x}", hash))
}
