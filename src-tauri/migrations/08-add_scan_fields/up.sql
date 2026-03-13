ALTER TABLE tracks ADD COLUMN file_size INTEGER;
ALTER TABLE tracks ADD COLUMN modified_time INTEGER;
ALTER TABLE tracks ADD COLUMN content_hash TEXT;
ALTER TABLE tracks ADD COLUMN scan_status INTEGER DEFAULT 1;

CREATE INDEX idx_tracks_file_path ON tracks(file_path);
CREATE INDEX idx_tracks_content_hash ON tracks(content_hash);
CREATE INDEX idx_tracks_scan_status ON tracks(scan_status);
CREATE INDEX idx_tracks_fingerprint ON tracks(modified_time, file_size);
