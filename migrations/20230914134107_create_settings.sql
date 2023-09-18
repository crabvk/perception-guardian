CREATE TABLE IF NOT EXISTS settings (
    chat_id INTEGER NOT NULL,
    setting_kind INTEGER NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (chat_id, setting_kind)
);
