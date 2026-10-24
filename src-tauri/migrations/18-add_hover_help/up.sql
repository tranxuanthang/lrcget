ALTER TABLE config_data ADD COLUMN show_hover_help INTEGER NOT NULL DEFAULT 0 CHECK (show_hover_help IN (0, 1));
