ALTER TABLE config_data ADD COLUMN spectrogram_min_hz REAL NOT NULL DEFAULT 62.5;
ALTER TABLE config_data ADD COLUMN spectrogram_max_hz REAL NOT NULL DEFAULT 8000;
ALTER TABLE config_data ADD COLUMN spectrogram_midpoint_db REAL NOT NULL DEFAULT -35;
