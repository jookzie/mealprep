//! Claude as the Mealprep planning assistant.
//!
//! One request, one answer: the user's correction or question, the conversation it
//! continues and a view of their plan go to the Messages API, and the reply comes back as tool calls — one per edit — plus some
//! prose. The tool calls become [`Operation`]s and the prose becomes the rationale, which
//! for a question is the whole answer; nothing here writes anything, and nothing here
//! decides whether an operation is allowed. `mealprep-core` checks all of that against
//! the real data.
//!
//! The key is the user's own, supplied per call by the rules, and the only place it goes
//! is the `x-api-key` header of this one request.

mod tool;

use std::time::Duration;

use mealprep_core::{
    Entity, Error, Result,
    domain::{AssistantContext, ChatTurn, Operation, Proposal, TurnOutcome},
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
You are the assistant of a meal-planning app. A request is either a change to make, which \
you turn into typed operations by calling the tools, or a question, which you answer in \
prose without calling any. An edit only happens through a tool call; prose never changes \
anything.

The user's data comes with the request as JSON: `today`, their daily macro `targets`, \
`days` (one entry per date you can see, with the plan on it, that plan's order, its macros \
and its `nutrients`), the `plans` (with their items), `meals` (with their servings) and \
`products` (with macros and `nutrients` per 100 g or ml) they have, each with its id, and \
the `categories` meals and plans are filed under.

Nutrients beyond the four macros are in grams, keyed by Open Food Facts name (`vitamin-c`, \
`iron`, `fiber`, `salt`, …). A day's `nutrients` sum only the products that state each one, \
so a figure is a floor, and a nutrient no product states is unknown rather than zero.

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
- Any field of a product, meal, day plan or the targets can be changed: a product's \
nutrients, brand, unit and price as well as its macros, and the category of a meal or plan. \
To sync, refresh or update a product from Open Food Facts or \"online\", call sync-product \
rather than giving figures yourself: the app fetches them.
- When the request cannot be expressed with these tools, or the context lacks what it \
needs, make no call and say in one sentence what is missing.

Questions — \"how much iron am I getting\", \"what's my vitamin D against the RDI\", \"which \
day is lowest in fibre\" — are answered from the context with no call. Compare \
micronutrients against the EU reference intakes for an adult (Regulation 1169/2011, Annex \
XIII) unless the user gives other figures, give amounts in the unit a label uses (mg, µg) \
with the share of the reference as a percentage, and say which nutrients are unknown or \
only partly known because products do not state them. For \"my\" figures say once that \
personal needs vary with age, sex and circumstances. Be factual: no diagnosis and no \
supplement advice. Unless the user names days, use today and the days after it that have \
a plan, and say which days you used.

Your text is shown to the user as plain text — no Markdown, no tables, no headings; use \
short lines and \"- \" for a list. For a change it sits above the change, for them to \
confirm or discard, so make it one or two plain sentences saying what you did. For a \
question it is the whole answer, so make it as long as the question needs and no longer. \
Either way use everyday words: no ids, no tool names. The app checks every operation \
against the real data and applies nothing without that confirmation, so propose what the \
request actually says rather than hedging.

The requests form a conversation, and a request may lean on an earlier one (\"make it 30 g \
instead\", \"and the day after too\"). Only the latest message carries the plan, and it is \
the plan as it is now: a change the user applied is already in it, and one they discarded \
never happened. A note in brackets at the start of a message says which it was. Act on the \
latest request only, and never propose again a change that was applied.";

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
        history: &[ChatTurn],
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
            .json(&body(history, request, context)?)
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

/// The request body: the system prompt, one tool per kind of operation, the conversation
/// so far, and the user's correction with the context it is about.
fn body(history: &[ChatTurn], request: &str, context: &AssistantContext) -> Result<Value> {
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
        "messages": messages(history, &format!("<plan>\n{context}\n</plan>\n\n{request}")),
    }))
}

/// The conversation as alternating turns, ending with `latest`.
///
/// Earlier turns are words only: the operations they proposed are not replayed, because
/// what was applied is in the plan and what was not never happened. What became of a
/// reply opens the message after it, where it reads as news rather than as something the
/// model said.
fn messages(history: &[ChatTurn], latest: &str) -> Vec<Value> {
    let mut messages = Vec::with_capacity(history.len() * 2 + 1);
    let mut note = None;
    for turn in history {
        messages.push(json!({ "role": "user", "content": noted(note, &turn.request) }));
        // A change can come with no prose, and the API refuses an empty turn.
        let reply = if turn.reply.is_empty() {
            "(I proposed a change.)"
        } else {
            turn.reply.as_str()
        };
        messages.push(json!({ "role": "assistant", "content": reply }));
        note = match turn.outcome {
            TurnOutcome::Answered => None,
            TurnOutcome::Applied => Some("[The user applied that change.]"),
            TurnOutcome::Discarded => {
                Some("[The user discarded that change; nothing was applied.]")
            }
        };
    }
    messages.push(json!({ "role": "user", "content": noted(note, latest) }));
    messages
}

fn noted(note: Option<&str>, text: &str) -> String {
    match note {
        Some(note) => format!("{note}\n\n{text}"),
        None => text.to_owned(),
    }
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
                brand: None,
                macros: Macros {
                    energy_kcal: 380.0,
                    fat_g: 6.0,
                    protein_g: 80.0,
                    carbohydrates_g: 8.0,
                },
                nutrients: [("calcium".to_owned(), 0.5)].into(),
                synced: true,
            }],
            categories: Vec::new(),
        }
    }

    #[test]
    fn the_request_carries_the_model_the_tools_and_the_plan() {
        let body = body(&[], "next 4 days I have 40 g less whey", &context()).unwrap();

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
        // The nutrients go too, so a question about them can be answered.
        assert!(content.contains("\"calcium\":0.5"), "{content}");
        // The key is a header, never part of the body.
        assert!(!body.to_string().contains("x-api-key"));
    }

    #[test]
    fn earlier_turns_go_first_and_only_the_latest_carries_the_plan() {
        let history = [
            ChatTurn {
                request: "next 4 days I have 40 g less whey".to_owned(),
                reply: "Took 40 g off the whey on each of the next four days.".to_owned(),
                outcome: TurnOutcome::Discarded,
            },
            ChatTurn {
                request: "how much calcium is that?".to_owned(),
                reply: "About 100 mg a day.".to_owned(),
                outcome: TurnOutcome::Answered,
            },
        ];
        let body = body(&history, "make it 30 g instead", &context()).unwrap();
        let messages = body["messages"].as_array().unwrap();

        let roles: Vec<&str> = messages
            .iter()
            .map(|message| message["role"].as_str().unwrap())
            .collect();
        assert_eq!(roles, ["user", "assistant", "user", "assistant", "user"]);
        assert_eq!(messages[0]["content"], "next 4 days I have 40 g less whey");

        // What became of a change opens the message after it.
        let after_discard = messages[2]["content"].as_str().unwrap();
        assert!(
            after_discard.starts_with("[The user discarded"),
            "{after_discard}"
        );
        assert!(after_discard.ends_with("how much calcium is that?"));

        // An answer leaves nothing to report, and the plan goes once, with the latest.
        let latest = messages[4]["content"].as_str().unwrap();
        assert!(latest.starts_with("<plan>"), "{latest}");
        assert!(latest.ends_with("make it 30 g instead"));
        assert_eq!(body.to_string().matches("<plan>").count(), 1);
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
