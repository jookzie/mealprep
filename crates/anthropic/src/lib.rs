//! Claude as the Mealprep planning assistant.
//!
//! One request, one answer: the user's correction and a compact view of their plan go to
//! the Messages API, and the reply comes back as tool calls — one per edit — plus a
//! sentence or two of prose. The tool calls become [`Operation`]s and the prose becomes
//! the rationale; nothing here writes anything, and nothing here decides whether an
//! operation is allowed. `mealprep-core` checks all of that against the real data.
//!
//! The key is the user's own, supplied per call by the rules, and the only place it goes
//! is the `x-api-key` header of this one request.

mod tool;

use std::time::Duration;

use mealprep_core::{
    Entity, Error, Result,
    domain::{AssistantContext, Operation, Proposal},
    store::Assistant,
};
use reqwest::{Client, StatusCode, header};
use serde::Deserialize;
use serde_json::{Value, json};

const BASE_URL: &str = "https://api.anthropic.com";

/// Claude Sonnet: the structured multi-step edits this asks for are beyond what a model
/// small enough to run on the phone does reliably, and beyond what the cheapest hosted
/// tier does either.
const MODEL: &str = "claude-sonnet-5";

/// The Messages API version header, which is not the model and does not move with it.
const API_VERSION: &str = "2023-06-01";

/// Room for the thinking the model does before the calls. The answer itself is a handful
/// of tool calls and two sentences, so this is a ceiling rather than a size.
const MAX_TOKENS: u32 = 16_000;

/// Long enough for a model that thinks before it answers; short enough that a phone with
/// no usable connection says so rather than hanging.
const TIMEOUT: Duration = Duration::from_secs(120);

/// What the model is for, and the rules it has to work inside.
///
/// The app's own constraints are stated here as well as enforced in `mealprep-core`:
/// a model that knows a past date will be refused proposes a workable edit instead of one
/// the user has to watch fail.
const SYSTEM: &str = "\
You turn one plain-language request about a meal plan into typed operations by calling the \
tools. An edit only happens through a tool call; prose never changes anything.

The user's data comes with the request as JSON: `today`, their daily macro `targets`, \
`days` (one entry per date you can see, with the plan on it and that plan's order), and the \
`plans` (with their items), `meals` (with their servings) and `products` (with macros per \
100 g or ml) they have, each with its id.

- Use only ids that appear in that context, or the newId of something a create call in \
the same reply makes. Never guess at something you cannot find by name — say what is \
missing instead.
- Prefer what exists: create a product, meal or day plan only when the request asks for \
something new or nothing in the context is it. A new product needs figures you can stand \
behind — the user's, or a typical package's for a common food.
- Updating a product, meal or day plan changes it everywhere it is used, past dates \
included. When the request is about some dates only (\"tomorrow I have no banana\"), edit \
those dates instead; update the thing itself only when the request is about it (\"my \
porridge has 60 g of oats now\").
- Resolve every relative date against `today` and give absolute YYYY-MM-DD dates. \"The \
next four days\" starts today.
- A date before `today` is refused by the app unless the user asked for a past day in so \
many words, so only name one when they did.
- One call per edit, and several calls in one reply when the request covers more than one \
thing. A run of consecutive dates is one call with a date range, not one call per day.
- A product can only be adjusted where it is an item of the day's own order. A product \
inside a meal belongs to that meal, which every other day on that meal shares.
- An item a day holds twice cannot be named: the app refuses it, because order is the only \
sequence a day carries. Say which two entries it could be instead of picking one.
- When the request cannot be expressed with these tools, or the context lacks what it \
needs, make no call and say in one sentence what is missing.

Your text reply is shown to the user above the change, for them to confirm or discard, so \
make it one or two plain sentences saying what you did, in everyday words: no ids, no tool \
names. The app checks every operation \
against the real data and applies nothing without that confirmation, so propose what the \
request actually says rather than hedging.";

/// Reads proposals from the Anthropic Messages API.
#[derive(Clone, Debug)]
pub struct Anthropic {
    http: Client,
    base_url: String,
}

impl Anthropic {
    /// Creates a client for the public Anthropic API.
    pub fn new() -> Result<Self> {
        Self::with_base_url(BASE_URL)
    }

    /// Creates a client for the given host, such as a gateway or a test double.
    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let http = Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(|source| Error::Internal {
                reason: format!("cannot build the Anthropic client: {source}"),
            })?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }
}

impl Assistant for Anthropic {
    async fn propose(
        &self,
        key: &str,
        request: &str,
        context: &AssistantContext,
    ) -> Result<Proposal> {
        let url = format!("{}/v1/messages", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .header(header::ACCEPT, "application/json")
            .json(&body(request, context)?)
            .send()
            .await
            .map_err(unavailable)?;

        let status = response.status();
        // Read as text either way: a failure answers with its own JSON shape, and a body
        // that is neither is worth quoting rather than discarding.
        let text = response.text().await.map_err(unavailable)?;
        if !status.is_success() {
            return Err(failed(status, &text));
        }

        let message: Message = serde_json::from_str(&text).map_err(|source| Error::Internal {
            reason: format!("cannot read what the assistant answered: {source}"),
        })?;
        proposal(request, message)
    }
}

/// The request body: the system prompt, one tool per kind of operation, and the user's
/// correction with the context it is about.
fn body(request: &str, context: &AssistantContext) -> Result<Value> {
    let context = serde_json::to_string(context).map_err(|source| Error::Internal {
        reason: format!("cannot describe the plan to the assistant: {source}"),
    })?;

    Ok(json!({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        // Adaptive is the only on-mode on this model, and resolving a sentence onto typed
        // edits over real ids is exactly the work that benefits from it.
        "thinking": { "type": "adaptive" },
        "system": SYSTEM,
        "tools": tool::tools(),
        // Auto rather than forced: "none of these tools can do that" is a useful answer,
        // and a forced call would have to invent something.
        "tool_choice": { "type": "auto" },
        "messages": [{
            "role": "user",
            "content": format!("<plan>\n{context}\n</plan>\n\n{request}"),
        }],
    }))
}

/// Reads the answer: the tool calls are the operations, the prose is the rationale.
fn proposal(request: &str, message: Message) -> Result<Proposal> {
    if message.stop_reason.as_deref() == Some("refusal") {
        let detail = message
            .stop_details
            .and_then(|details| details.explanation)
            .unwrap_or_else(|| "it gave no reason".to_owned());
        return Err(Error::Invalid {
            entity: Entity::AssistantRequest,
            reason: format!("the assistant declined this request: {detail}"),
        });
    }
    if message.stop_reason.as_deref() == Some("max_tokens") {
        return Err(Error::AssistantUnavailable {
            reason: "the answer was cut off before it finished; try asking for one change"
                .to_owned(),
        });
    }

    let mut prose: Vec<String> = Vec::new();
    let mut operations: Vec<Operation> = Vec::new();
    for block in message.content {
        match block {
            Block::Text { text } if !text.trim().is_empty() => prose.push(text.trim().to_owned()),
            Block::Text { .. } => {}
            Block::ToolUse { name, input } => operations.push(tool::operation(&name, input)?),
            // Thinking blocks and anything a later API version adds: not ours to read.
            Block::Other => {}
        }
    }

    let rationale = prose.join("\n\n");
    if operations.is_empty() && rationale.is_empty() {
        return Err(Error::Invalid {
            entity: Entity::AssistantRequest,
            reason: "the assistant proposed nothing and gave no reason".to_owned(),
        });
    }

    Ok(Proposal {
        // Proposing nothing with a reason is a valid answer, and reads better than a
        // guess: the preview is then empty and the user sees why.
        request: request.to_owned(),
        rationale,
        operations,
    })
}

/// A failed call, by what the user can do about it.
fn failed(status: StatusCode, body: &str) -> Error {
    let detail = serde_json::from_str::<ApiError>(body)
        .map(|body| body.error.message)
        .unwrap_or_else(|_| body.chars().take(200).collect());

    match status {
        // The key is the one thing here the user holds, so this is theirs to fix.
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Error::Invalid {
            entity: Entity::AssistantRequest,
            reason: format!("the API key was refused: {detail}"),
        },
        StatusCode::TOO_MANY_REQUESTS => Error::AssistantUnavailable {
            reason: format!("too many requests just now: {detail}"),
        },
        status if status.is_server_error() => Error::AssistantUnavailable {
            reason: format!("the assistant answered {status}: {detail}"),
        },
        // Any other 4xx is this crate sending the wrong request, which is not the user's
        // to act on.
        status => Error::Internal {
            reason: format!("the assistant rejected the request ({status}): {detail}"),
        },
    }
}

fn unavailable(source: reqwest::Error) -> Error {
    Error::AssistantUnavailable {
        reason: source.to_string(),
    }
}

// serde types

#[derive(Deserialize)]
struct Message {
    #[serde(default)]
    content: Vec<Block>,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    stop_details: Option<StopDetails>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text {
        text: String,
    },
    ToolUse {
        name: String,
        input: Value,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct StopDetails {
    #[serde(default)]
    explanation: Option<String>,
}

#[derive(Deserialize)]
struct ApiError {
    error: ApiErrorBody,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    message: String,
}

#[cfg(test)]
mod tests {
    use mealprep_core::domain::{DateRange, Macros, Unit, parse_iso_date};
    use uuid::uuid;

    use super::*;

    /// Recorded answers. No live call is made by this crate's tests: the request shape and
    /// the mapping back are what is under test, and both are pure.
    const PROPOSAL: &str = include_str!("../fixtures/proposal.json");
    const NOTHING_TO_DO: &str = include_str!("../fixtures/nothing-to-do.json");
    const REFUSAL: &str = include_str!("../fixtures/refusal.json");
    const CUT_OFF: &str = include_str!("../fixtures/cut-off.json");
    const UNAUTHORIZED: &str = include_str!("../fixtures/unauthorized.json");

    fn parse(fixture: &str) -> Message {
        serde_json::from_str(fixture).expect("the fixture is an answer from the API")
    }

    fn context() -> AssistantContext {
        AssistantContext {
            today: parse_iso_date("2026-10-20").unwrap(),
            targets: Some(Macros {
                energy_kcal: 2000.0,
                fat_g: 65.0,
                protein_g: 140.0,
                carbohydrates_g: 220.0,
            }),
            days: Vec::new(),
            plans: Vec::new(),
            meals: Vec::new(),
            products: vec![mealprep_core::domain::ContextProduct {
                id: uuid!("3f9a1c52-0000-4000-8000-000000000001"),
                name: "Whey".to_owned(),
                unit: Unit::Gram,
                macros: Macros {
                    energy_kcal: 380.0,
                    fat_g: 6.0,
                    protein_g: 80.0,
                    carbohydrates_g: 8.0,
                },
            }],
        }
    }

    #[test]
    fn the_request_carries_the_model_the_tools_and_the_plan() {
        let body = body("next 4 days I have 40 g less whey", &context()).unwrap();

        assert_eq!(body["model"], MODEL);
        assert_eq!(body["thinking"]["type"], "adaptive");
        assert_eq!(body["tool_choice"]["type"], "auto");
        assert_eq!(
            body["tools"].as_array().unwrap().len(),
            mealprep_core::domain::OperationKind::ALL.len()
        );

        let content = body["messages"][0]["content"].as_str().unwrap();
        assert!(content.ends_with("next 4 days I have 40 g less whey"));
        assert!(content.contains("\"today\":\"2026-10-20\""), "{content}");
        assert!(content.contains("Whey"), "{content}");
        // The key is a header, never part of the body.
        assert!(!body.to_string().contains("x-api-key"));
    }

    #[test]
    fn tool_calls_become_operations_and_the_prose_becomes_the_rationale() {
        let proposal = proposal("next 4 days I have 40 g less whey", parse(PROPOSAL)).unwrap();

        assert_eq!(proposal.request, "next 4 days I have 40 g less whey");
        assert_eq!(
            proposal.rationale,
            "Took 40 g off the scoop of whey on each of the next four days, leaving 20 g."
        );
        assert_eq!(
            proposal.operations,
            vec![Operation::AdjustProductAmount {
                dates: DateRange {
                    from: parse_iso_date("2026-10-20").unwrap(),
                    to: parse_iso_date("2026-10-23").unwrap(),
                },
                product_id: uuid!("3f9a1c52-0000-4000-8000-000000000001"),
                delta_amount: -40.0,
            }]
        );
    }

    #[test]
    fn proposing_nothing_with_a_reason_is_a_valid_answer() {
        let proposal = proposal("make it healthier", parse(NOTHING_TO_DO)).unwrap();
        assert!(proposal.operations.is_empty());
        assert!(proposal.rationale.contains("which product"));
    }

    #[test]
    fn a_refusal_and_a_cut_off_answer_are_failures_rather_than_empty_proposals() {
        let Err(Error::Invalid { reason, .. }) = proposal("...", parse(REFUSAL)) else {
            panic!("a refusal is not a proposal");
        };
        assert!(reason.contains("declined"), "{reason}");

        assert!(matches!(
            proposal("...", parse(CUT_OFF)),
            Err(Error::AssistantUnavailable { .. })
        ));
    }

    #[test]
    fn a_failed_call_is_sorted_by_what_the_user_can_do_about_it() {
        // A refused key is the user's to fix, so it reads as invalid rather than internal.
        let Error::Invalid { reason, .. } = failed(StatusCode::UNAUTHORIZED, UNAUTHORIZED) else {
            panic!("a refused key is the user's to fix");
        };
        assert!(reason.contains("invalid x-api-key"), "{reason}");

        assert!(matches!(
            failed(StatusCode::TOO_MANY_REQUESTS, UNAUTHORIZED),
            Error::AssistantUnavailable { .. }
        ));
        assert!(matches!(
            failed(StatusCode::INTERNAL_SERVER_ERROR, "<html>down</html>"),
            Error::AssistantUnavailable { .. }
        ));
        // Our own mistake, not theirs.
        assert!(matches!(
            failed(StatusCode::BAD_REQUEST, "{}"),
            Error::Internal { .. }
        ));
    }
}
