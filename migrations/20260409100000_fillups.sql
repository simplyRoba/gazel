CREATE TABLE fillups (
    id           INTEGER PRIMARY KEY,
    vehicle_id   INTEGER NOT NULL REFERENCES vehicles(id),
    date         TEXT    NOT NULL,
    odometer     REAL,
    fuel_amount  REAL    NOT NULL,
    fuel_unit    TEXT    NOT NULL DEFAULT 'liters',
    cost         REAL,
    currency     TEXT,
    is_full_tank INTEGER NOT NULL DEFAULT 0,
    is_missed    INTEGER NOT NULL DEFAULT 0,
    station      TEXT,
    notes        TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_fillups_vehicle_id ON fillups(vehicle_id);
CREATE INDEX idx_fillups_date ON fillups(vehicle_id, date DESC);
