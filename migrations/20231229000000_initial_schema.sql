-- Create manga table
CREATE TABLE IF NOT EXISTS manga (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    cover TEXT NOT NULL,
    cover_small TEXT NOT NULL
);

-- Create website table
CREATE TABLE IF NOT EXISTS website (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT UNIQUE NOT NULL
);

-- Create source table
CREATE TABLE IF NOT EXISTS source (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    manga_id INTEGER NOT NULL,
    website_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE,
    FOREIGN KEY (website_id) REFERENCES website(id),
    UNIQUE (website_id, path)
);

-- Create chapter table
CREATE TABLE IF NOT EXISTS chapter (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    manga_id INTEGER NOT NULL,
    number TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE
);

-- Create setting table
CREATE TABLE IF NOT EXISTS setting (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Trigger to auto-update updated_at in chapter table
CREATE TRIGGER IF NOT EXISTS update_chapter_updated_at
AFTER UPDATE ON chapter
FOR EACH ROW
BEGIN
    UPDATE chapter SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
