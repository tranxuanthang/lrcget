use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use tauri::{AppHandle, Manager};
use tokio::process::Command;
use xxhash_rust::xxh3::xxh3_64;

use crate::scanner::hasher::compute_quick_hash;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;
const VOCALS_WAV: &str = "vocals.wav";

struct JobGuard(String);

impl Drop for JobGuard {
    fn drop(&mut self) {
        if let Ok(mut jobs) = running_jobs().lock() {
            jobs.remove(&self.0);
        }
    }
}

fn running_jobs() -> &'static Mutex<HashSet<String>> {
    static JOBS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn strip_windows_extended_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let value = path.to_string_lossy();
        if let Some(stripped) = value.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    path
}

fn normalize_source_path(file_path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(file_path);
    if !path.is_file() {
        return Err("Audio file not found".to_string());
    }

    let canonical = std::fs::canonicalize(&path).unwrap_or(path);
    Ok(strip_windows_extended_prefix(canonical))
}

fn cache_key(source_path: &Path) -> String {
    let hash = compute_quick_hash(source_path).unwrap_or_else(|_| {
        format!("{:016x}", xxh3_64(source_path.to_string_lossy().as_bytes()))
    });
    let size = std::fs::metadata(source_path)
        .map(|meta| meta.len())
        .unwrap_or(0);
    let modified = std::fs::metadata(source_path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    format!("{hash}-{size}-{modified}")
}

fn vocal_stem_cache_dir(app_handle: &AppHandle, source_path: &Path) -> Result<PathBuf, String> {
    let cache_root = app_handle
        .path()
        .app_cache_dir()
        .map_err(|err| format!("Failed to resolve app cache directory: {err}"))?;
    Ok(cache_root.join("vocal-stems").join(cache_key(source_path)))
}

pub fn expected_cached_vocals_path(out_dir: &Path) -> PathBuf {
    out_dir.join("htdemucs").join(VOCALS_WAV)
}

fn find_vocals_wav(root: &Path) -> Option<PathBuf> {
    if !root.is_dir() {
        return None;
    }

    let expected = expected_cached_vocals_path(root);
    if expected.is_file() {
        return Some(expected);
    }

    walk_for_vocals_wav(root, 0)
}

fn walk_for_vocals_wav(dir: &Path, depth: u32) -> Option<PathBuf> {
    if depth > 4 {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(OsStr::to_str) == Some(VOCALS_WAV) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = walk_for_vocals_wav(&path, depth + 1) {
                return Some(found);
            }
        }
    }

    None
}

pub fn find_sidecar_vocals(source_path: &Path) -> Option<PathBuf> {
    let parent = source_path.parent()?;
    let stem = source_path.file_stem()?;
    let separated = parent.join("separated");
    if !separated.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(&separated).ok()?;
    for entry in entries.flatten() {
        let model_dir = entry.path();
        if !model_dir.is_dir() {
            continue;
        }
        let candidate = model_dir.join(stem).join(VOCALS_WAV);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

pub fn find_existing_vocal_stem(app_handle: &AppHandle, file_path: &str) -> Result<Option<PathBuf>, String> {
    let source_path = normalize_source_path(file_path)?;
    let cache_dir = vocal_stem_cache_dir(app_handle, &source_path)?;

    if let Some(cached) = find_vocals_wav(&cache_dir) {
        return Ok(Some(cached));
    }

    Ok(find_sidecar_vocals(&source_path))
}

fn try_claim_job(key: &str) -> Result<JobGuard, String> {
    let mut jobs = running_jobs()
        .lock()
        .map_err(|_| "Failed to lock vocal separation jobs".to_string())?;
    if !jobs.insert(key.to_string()) {
        return Err("Vocal separation is already running for this track".to_string());
    }
    Ok(JobGuard(key.to_string()))
}

fn is_not_found(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("cannot find")
        || lower.contains("not found")
        || lower.contains("os error 2")
        || lower.contains("program not found")
        || lower.contains("no such file")
}

async fn run_demucs_command(
    program: &str,
    prefix_args: &[&str],
    out_dir: &Path,
    source_path: &Path,
) -> Result<(), String> {
    let mut cmd = Command::new(program);
    cmd.args(prefix_args)
        .arg("--shifts=4")
        .arg("--two-stems=vocals")
        .arg("-o")
        .arg(out_dir)
        .arg(source_path);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .await
        .map_err(|err| format!("Failed to run {program}: {err}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let details = if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };

    Err(if details.is_empty() {
        format!("{program} failed with status {}", output.status)
    } else {
        format!("{program} failed: {details}")
    })
}

async fn run_demucs(out_dir: &Path, source_path: &Path) -> Result<(), String> {
    let attempts: &[(&str, &[&str])] = &[
        ("demucs", &[]),
        ("python", &["-m", "demucs"]),
        ("python3", &["-m", "demucs"]),
        ("py", &["-m", "demucs"]),
    ];

    let mut last_error = String::from("Demucs is not installed or not on PATH");
    for (program, prefix_args) in attempts {
        match run_demucs_command(program, prefix_args, out_dir, source_path).await {
            Ok(()) => return Ok(()),
            Err(err) if is_not_found(&err) => {
                last_error = err;
            }
            Err(err) => return Err(err),
        }
    }

    Err(format!(
        "{last_error}. Install Demucs and ensure `demucs` is on PATH."
    ))
}

pub async fn separate_vocals(app_handle: &AppHandle, file_path: &str) -> Result<PathBuf, String> {
    if let Some(existing) = find_existing_vocal_stem(app_handle, file_path)? {
        return Ok(existing);
    }

    let source_path = normalize_source_path(file_path)?;
    let out_dir = vocal_stem_cache_dir(app_handle, &source_path)?;
    std::fs::create_dir_all(&out_dir)
        .map_err(|err| format!("Failed to create vocal stem cache: {err}"))?;

    let job_key = out_dir.to_string_lossy().to_string();
    let _guard = try_claim_job(&job_key)?;

    if let Some(existing) = find_vocals_wav(&out_dir) {
        return Ok(existing);
    }

    run_demucs(&out_dir, &source_path).await?;

    find_vocals_wav(&out_dir).ok_or_else(|| {
        "Demucs finished but vocals.wav was not found in the output folder".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn write_file(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = fs::File::create(path).unwrap();
        file.write_all(b"vocals").unwrap();
    }

    #[test]
    fn expected_cached_path_uses_htdemucs_vocals_wav() {
        let out_dir = PathBuf::from("vocal-stems").join("abc");
        assert_eq!(
            expected_cached_vocals_path(&out_dir),
            PathBuf::from("vocal-stems")
                .join("abc")
                .join("htdemucs")
                .join("vocals.wav")
        );
    }

    #[test]
    fn find_vocals_wav_prefers_htdemucs_layout() {
        let tmp = std::env::temp_dir().join(format!(
            "lrcget-vocal-stem-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        let expected = expected_cached_vocals_path(&tmp);
        write_file(&expected);
        let nested = tmp.join("other").join(VOCALS_WAV);
        write_file(&nested);

        assert_eq!(find_vocals_wav(&tmp), Some(expected));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn find_sidecar_vocals_looks_in_separated_model_folder() {
        let tmp = std::env::temp_dir().join(format!(
            "lrcget-vocal-stem-sidecar-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let source = tmp.join("song.mp3");
        write_file(&source);
        let vocals = tmp
            .join("separated")
            .join("htdemucs")
            .join("song")
            .join(VOCALS_WAV);
        write_file(&vocals);

        assert_eq!(find_sidecar_vocals(&source), Some(vocals));
        let _ = fs::remove_dir_all(&tmp);
    }
}
