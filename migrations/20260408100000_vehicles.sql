CREATE TABLE vehicles (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    make       TEXT,
    model      TEXT,
    year       INTEGER,
    fuel_type  TEXT    NOT NULL DEFAULT 'gasoline',
    notes      TEXT,
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
