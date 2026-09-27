-- One weigh-in per date, keyed by the date the way calendar_days is: a day has one
-- number worth keeping, and logging again replaces it.
CREATE TABLE weight_entries (
    date       TEXT PRIMARY KEY,
    kilograms  REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);
