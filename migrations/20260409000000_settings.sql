CREATE TABLE settings (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    unit_system   TEXT NOT NULL DEFAULT 'metric',
    distance_unit TEXT NOT NULL DEFAULT 'km',
    volume_unit   TEXT NOT NULL DEFAULT 'l',
    currency      TEXT NOT NULL DEFAULT 'USD',
    color_mode    TEXT NOT NULL DEFAULT 'system',
    locale        TEXT NOT NULL DEFAULT 'en'
);

INSERT INTO settings (id) VALUES (1);
