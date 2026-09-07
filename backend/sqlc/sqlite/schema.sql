-- Every table carries created_at, updated_at and deleted_at (RFC 3339 text).
-- Rows are never removed; a delete sets deleted_at.

CREATE TABLE IF NOT EXISTS products (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    unit            TEXT NOT NULL CHECK (unit IN ('g', 'ml')),
    energy_kcal     REAL NOT NULL,
    fat_g           REAL NOT NULL,
    protein_g       REAL NOT NULL,
    carbohydrates_g REAL NOT NULL,
    nutrients       TEXT NOT NULL,
    brand           TEXT,
    source_code     TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);

CREATE TABLE IF NOT EXISTS meals (
    id         TEXT PRIMARY KEY,
    label      TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS meal_servings (
    id         TEXT PRIMARY KEY,
    meal_id    TEXT NOT NULL REFERENCES meals (id),
    product_id TEXT NOT NULL REFERENCES products (id),
    amount     REAL NOT NULL,
    position   INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS day_plans (
    id         TEXT PRIMARY KEY,
    label      TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS day_plan_meals (
    id          TEXT PRIMARY KEY,
    day_plan_id TEXT NOT NULL REFERENCES day_plans (id),
    meal_id     TEXT NOT NULL REFERENCES meals (id),
    position    INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE IF NOT EXISTS calendar_days (
    date        TEXT PRIMARY KEY,
    day_plan_id TEXT NOT NULL REFERENCES day_plans (id),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE IF NOT EXISTS targets (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    energy_kcal     REAL NOT NULL,
    fat_g           REAL NOT NULL,
    protein_g       REAL NOT NULL,
    carbohydrates_g REAL NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);
