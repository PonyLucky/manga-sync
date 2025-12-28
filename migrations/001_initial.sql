-- Création des tables

CREATE TABLE manga (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    cover TEXT,
    cover_small TEXT
);

CREATE TABLE website (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT NOT NULL UNIQUE
);

CREATE TABLE source (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    manga_id INTEGER NOT NULL,
    website_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE,
    FOREIGN KEY (website_id) REFERENCES website(id),
    UNIQUE (website_id, path)
);

CREATE TABLE chapter (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    manga_id INTEGER NOT NULL,
    number TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE
);

CREATE TABLE setting (
    key TEXT PRIMARY KEY,
    value TEXT
);
