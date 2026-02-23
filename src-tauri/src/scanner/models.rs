/// Track information from the database
#[derive(Debug, Clone)]
pub struct DbTrack {
    pub id: i64,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub modified_time: Option<i64>,
    pub content_hash: Option<String>,
}

/// Summary of scan operation results
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// Total files found on disk
    pub total_files: usize,
    /// Number of new files added
    pub added: usize,
    /// Number of files modified
    pub modified: usize,
    /// Number of files deleted
    pub deleted: usize,
    /// Number of files moved/renamed
    pub moved: usize,
    /// Number of unchanged files
    pub unchanged: usize,
    /// Whether this was the first successful scan
    pub is_initial_scan: bool,
    /// Duration of scan in milliseconds
    pub duration_ms: u64,
}

/// Progress update during scan
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    /// Current phase: "processing" or "updating"
    pub phase: String,
    /// Current progress (0.0 to 1.0) within this phase
    pub progress: f64,
    /// Number of files processed so far
    pub files_processed: usize,
    /// Total number of files to process (if known)
    pub files_total: Option<usize>,
    /// Human-readable status message
    pub message: String,
}

impl ScanProgress {
    pub fn new(
        phase: &str,
        progress: f64,
        files_processed: usize,
        files_total: Option<usize>,
        message: &str,
    ) -> Self {
        Self {
            phase: phase.to_string(),
            progress,
            files_processed,
            files_total,
            message: message.to_string(),
        }
    }

    pub fn processing(processed: usize, total: usize) -> Self {
        let progress = if total > 0 {
            processed as f64 / total as f64
        } else {
            0.0
        };
        Self::new(
            "processing",
            progress,
            processed,
            Some(total),
            &format!("Processing files: {}", processed),
        )
    }

    pub fn updating() -> Self {
        Self::new("updating", 0.0, 0, None, "Updating database...")
    }
}
