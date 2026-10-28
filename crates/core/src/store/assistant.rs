use crate::{
    Result,
    domain::{
        AssistantContext, AssistantRecord, AssistantRecordDraft, ChatTurn, Operation, Proposal,
    },
};

/// Whatever turns a plain-language request into typed operations.
///
/// The implementation reaches a model; which one, and where it runs, `core` does not know
/// and must not learn. What it has to keep is the contract below, because the rules trust
/// nothing it returns:
///
/// - it writes nothing, anywhere. It answers with a [`Proposal`], which the rules then
///   check against the real data before the user is shown anything;
/// - it invents no ids. Every id in the proposal comes from the context it was given;
///   one that does not is caught as [`Error::Invalid`](crate::Error::Invalid);
/// - it is free to propose nothing. An empty operation list with a rationale saying why is
///   a valid answer, and reads better than a guess.
#[trait_variant::make(Send)]
pub trait Assistant {
    /// Proposes the operations that would satisfy `request`, given `context`.
    ///
    /// `history` is the conversation so far, oldest first, which the request may lean on
    /// ("make it 30 g instead"). `context` is the plan as it is now, after whatever those
    /// turns applied.
    ///
    /// `key` is the credential the user supplied for whichever service the implementation
    /// talks to. `core` never looks at it; it reads it from [`AssistantStore`] and hands
    /// it over, so no implementation has to carry state the rules cannot see.
    ///
    /// `check` runs the rules over a set of operations and says why they would be refused.
    /// An implementation may call it on an answer and, when it fails, ask again with that
    /// reason, a bounded number of times: a request that takes several edits fails as a
    /// whole on any one of them, and the reason is usually enough to put that one right.
    /// Passing it is no promise — the rules check whatever comes back from scratch.
    async fn propose(
        &self,
        key: &str,
        history: &[ChatTurn],
        request: &str,
        context: &AssistantContext,
        check: impl Fn(&[Operation]) -> Result<()> + Send + Sync,
    ) -> Result<Proposal>;
}

/// Persistence of the assistant's credential and of what it has been asked to do.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait AssistantStore {
    /// Fetches the credential the user pasted in, or `None` when there is none.
    ///
    /// It is stored as the user gave it and is never returned to the frontend in full.
    async fn get_assistant_key(&self) -> Result<Option<String>>;

    /// Replaces the stored credential.
    async fn set_assistant_key(&self, key: &str) -> Result<()>;

    /// Forgets the stored credential, whether or not there was one.
    async fn clear_assistant_key(&self) -> Result<()>;

    /// Records one applied request. Nothing records a request that was not applied.
    async fn record_assistant_request(
        &self,
        draft: &AssistantRecordDraft,
    ) -> Result<AssistantRecord>;

    /// Lists at most `limit` applied requests, most recent first.
    async fn list_assistant_requests(&self, limit: u32) -> Result<Vec<AssistantRecord>>;
}
