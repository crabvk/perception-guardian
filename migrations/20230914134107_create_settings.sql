CREATE TABLE settings (
    chat_id INTEGER PRIMARY KEY,
    language TEXT NOT NULL,
    ban_channels INTEGER,
    captcha_expire INTEGER NOT NULL,
    message_expire INTEGER NOT NULL,
    ignore_expire INTEGER NOT NULL,
    delete_entry_messages BOOLEAN NOT NULL
);
