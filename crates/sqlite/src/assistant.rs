use mealprep_core::{
    Entity, Error, Result,
    domain::{AppliedChange, AssistantRecord, AssistantRecordDraft, Operation},
    store::AssistantStore,
};
use uuid::Uuid;

use crate::{
    SqliteStore,
    row::{self, internal},
};

/// Where the assistant's credential is kept. One `app_settings` row, stored as the user
/// pasted it: it is read only to be put in a request header, and nothing else reads it.
const KEY_SETTING: &str = "assistant.api-key";

const GET_SETTING: &str = "SELECT value FROM app_settings WHERE key = ? AND deleted_at IS NULL";
const SET_SETTING: &str = "INSERT INTO app_settings (key, value, created_at, updated_at) \
    VALUES (?, ?, ?, ?) \
    ON CONFLICT (key) DO UPDATE \
    SET value = excluded.value, updated_at = excluded.updated_at, deleted_at = NULL";
const CLEAR_SETTING: &str = "UPDATE app_settings SET deleted_at = ?, updated_at = ? \
    WHERE key = ? AND deleted_at IS NULL";

const RECORD: &str = "INSERT INTO assistant_requests \
    (id, request, rationale, operations, changes, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *";
const LIST: &str = "SELECT * FROM assistant_requests \
    WHERE deleted_at IS NULL ORDER BY created_at DESC, id LIMIT ?";

impl AssistantStore for SqliteStore {
    async fn get_assistant_key(&self) -> Result<Option<String>> {
        let key: Option<String> = sqlx::query_scalar(GET_SETTING)
            .bind(KEY_SETTING)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        Ok(key)
    }

    async fn set_assistant_key(&self, key: &str) -> Result<()> {
        let now = row::now();
        sqlx::query(SET_SETTING)
            .bind(KEY_SETTING)
            .bind(key)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn clear_assistant_key(&self) -> Result<()> {
        let now = row::now();
        // Clearing a key that was never set is not a failure: the end state is the same.
        sqlx::query(CLEAR_SETTING)
            .bind(&now)
            .bind(&now)
            .bind(KEY_SETTING)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn record_assistant_request(
        &self,
        draft: &AssistantRecordDraft,
    ) -> Result<AssistantRecord> {
        let now = row::now();
        let record: RequestRow = sqlx::query_as(RECORD)
            .bind(Uuid::new_v4().to_string())
            .bind(&draft.request)
            .bind(&draft.rationale)
            .bind(encode_operations(&draft.operations)?)
            .bind(encode_changes(&draft.changes)?)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        record.into_record()
    }

    async fn list_assistant_requests(&self, limit: u32) -> Result<Vec<AssistantRecord>> {
        let records: Vec<RequestRow> = sqlx::query_as(LIST)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        records.into_iter().map(RequestRow::into_record).collect()
    }
}

#[derive(sqlx::FromRow)]
struct RequestRow {
    id: String,
    request: String,
    rationale: String,
    operations: String,
    changes: String,
    created_at: String,
}

impl RequestRow {
    fn into_record(self) -> Result<AssistantRecord> {
        let Self {
            id,
            request,
            rationale,
            operations,
            changes,
            created_at,
        } = self;

        let operations: Vec<Operation> = serde_json::from_str(&operations).map_err(internal)?;
        let changes: Vec<AppliedChange> = serde_json::from_str(&changes).map_err(internal)?;

        Ok(AssistantRecord {
            id: row::parse_id(&id)?,
            request,
            rationale,
            operations,
            changes,
            created_at: row::parse_timestamp(&created_at)?,
        })
    }
}

/// Stores the operations as the JSON the domain itself serialises.
///
/// The log is read back and shown, never queried by its parts, so one column of JSON is
/// the whole of what it needs — and it keeps its shape in step with the domain for free.
fn encode_operations(operations: &[Operation]) -> Result<String> {
    serde_json::to_string(operations).map_err(unrecordable)
}

fn encode_changes(changes: &[AppliedChange]) -> Result<String> {
    serde_json::to_string(changes).map_err(unrecordable)
}

fn unrecordable(source: serde_json::Error) -> Error {
    Error::Internal {
        reason: format!(
            "cannot record what the {} did: {source}",
            Entity::AssistantRequest
        ),
    }
}
