-- One value per kind per date, keyed the way weight_entries is: measuring again corrects
-- rather than accumulates.
CREATE TABLE measurements (
    date       TEXT NOT NULL,
    kind       TEXT NOT NULL,
    value      REAL NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    PRIMARY KEY (date, kind)
);

-- One row per morning, summarised from what Health Connect held when it was last read.
-- Every figure is nullable because every source is optional. Sleep times are minutes from
-- the morning's midnight, negative for the evening before.
CREATE TABLE health_days (
    date              TEXT PRIMARY KEY,
    bed_minute        INTEGER,
    wake_minute       INTEGER,
    asleep_minutes    REAL,
    light_minutes     REAL,
    deep_minutes      REAL,
    rem_minutes       REAL,
    awake_minutes     REAL,
    resting_heart_rate REAL,
    hrv_ms            REAL,
    respiratory_rate  REAL,
    active_kcal       REAL,
    total_kcal        REAL,
    exercise_minutes  REAL,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    deleted_at        TEXT
);

-- The last import, a singleton like targets.
CREATE TABLE health_sync (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    synced_at  TEXT NOT NULL,
    through    TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);
