-- Every table carries created_at, updated_at and deleted_at as RFC 3339 text.
-- Rows are never removed; a delete sets deleted_at.

CREATE TABLE products (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    unit            TEXT NOT NULL CHECK (unit IN ('g', 'ml')),
    energy_kcal     REAL NOT NULL,
    fat_g           REAL NOT NULL,
    protein_g       REAL NOT NULL,
    carbohydrates_g REAL NOT NULL,
    nutrients       TEXT NOT NULL,
    -- NULL rather than 0 when never entered: a sum has to tell unpriced from free.
    cost            REAL,
    brand           TEXT,
    source_code     TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);

CREATE TABLE categories (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    scope      TEXT NOT NULL CHECK (scope IN ('meal', 'day-plan')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE meals (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    category_id TEXT REFERENCES categories (id),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE meal_servings (
    id         TEXT PRIMARY KEY,
    meal_id    TEXT NOT NULL REFERENCES meals (id),
    product_id TEXT NOT NULL REFERENCES products (id),
    amount     REAL NOT NULL,
    position   INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX meal_servings_by_meal ON meal_servings (meal_id, position) WHERE deleted_at IS NULL;

CREATE TABLE day_plans (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    category_id TEXT REFERENCES categories (id),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

-- One row per entry in a plan's order. kind says which of meal_id and
-- (product_id, amount) is set; the other is NULL.
CREATE TABLE day_plan_items (
    id          TEXT PRIMARY KEY,
    day_plan_id TEXT NOT NULL REFERENCES day_plans (id),
    kind        TEXT NOT NULL CHECK (kind IN ('meal', 'product')),
    meal_id     TEXT REFERENCES meals (id),
    product_id  TEXT REFERENCES products (id),
    amount      REAL,
    position    INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT,
    CHECK (
        (kind = 'meal' AND meal_id IS NOT NULL AND product_id IS NULL AND amount IS NULL)
        OR (kind = 'product' AND meal_id IS NULL AND product_id IS NOT NULL AND amount IS NOT NULL)
    )
);

CREATE INDEX day_plan_items_by_plan ON day_plan_items (day_plan_id, position) WHERE deleted_at IS NULL;

CREATE TABLE calendar_days (
    date        TEXT PRIMARY KEY,
    day_plan_id TEXT NOT NULL REFERENCES day_plans (id),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

CREATE TABLE targets (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    energy_kcal     REAL NOT NULL,
    fat_g           REAL NOT NULL,
    protein_g       REAL NOT NULL,
    carbohydrates_g REAL NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);
