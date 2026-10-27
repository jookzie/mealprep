-- The products, meals and day plans an applied assistant request created or changed, as
-- JSON in the shape the domain serialises Vec<AppliedEntityChange>. Additive: a request
-- recorded before the assistant could edit them reads as having changed none.
ALTER TABLE assistant_requests ADD COLUMN entity_changes TEXT NOT NULL DEFAULT '[]';
