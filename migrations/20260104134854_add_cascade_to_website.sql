-- Add ON DELETE CASCADE to website_id in source table
-- SQLite doesn't support ALTER TABLE to add foreign key constraints, 
-- so we need to recreate the table.

PRAGMA foreign_keys=OFF;

CREATE TABLE source_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    manga_id INTEGER NOT NULL,
    website_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE,
    FOREIGN KEY (website_id) REFERENCES website(id) ON DELETE CASCADE,
    UNIQUE (website_id, path)
);

INSERT INTO source_new (id, manga_id, website_id, path)
SELECT id, manga_id, website_id, path FROM source;

DROP TABLE source;

ALTER TABLE source_new RENAME TO source;

PRAGMA foreign_keys=ON;
