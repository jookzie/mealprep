-- One date's own copy of a day plan. A calendar day points at a shared day_plans row, so
-- editing "the next four days" in place would rewrite every past date on that plan too.
-- A dated edit therefore copies the plan for the date it touches and points that date at
-- the copy; source_plan_id says which plan it came from. Copies are kept out of the plan
-- list and the pickers, so nothing but their own date can ever reach them.
ALTER TABLE day_plans ADD COLUMN source_plan_id TEXT REFERENCES day_plans (id);

-- Settings the user supplies that are not an entity of their own. One row today: the
-- assistant's API key, which the user pastes in and which never leaves the device.
CREATE TABLE app_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- One row per applied assistant request: what was asked, what the model said, the typed
-- operations that were applied, and what each date pointed at before. Enough to read what
-- the assistant did and put it back by hand. Nothing records a proposal that was not
-- applied.
CREATE TABLE assistant_requests (
    id         TEXT PRIMARY KEY,
    request    TEXT NOT NULL,
    rationale  TEXT NOT NULL,
    -- JSON: the Vec<Operation> and the Vec<AppliedChange> as the domain serialises them.
    operations TEXT NOT NULL,
    changes    TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX assistant_requests_by_time ON assistant_requests (created_at DESC) WHERE deleted_at IS NULL;
