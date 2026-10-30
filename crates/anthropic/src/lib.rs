//! Claude as the Mealprep planning assistant.
//!
//! The user's correction or question, the conversation it continues and a view of their
//! plan go to the Messages API, and the reply comes back as tool calls — one per edit —
//! plus some prose. The tool calls become [`Operation`]s and the prose becomes the
//! rationale, which for a question is the whole answer; nothing here writes anything, and
//! nothing here decides whether an operation is allowed. `mealprep-core` checks all of
//! that against the real data.
//!
//! An answer the rules would refuse goes back to the model once or twice with the reason,
//! as the error results of its calls, so a request that takes several edits is not lost
//! to one of them. The tool calls are never run here: they are only ever proposals.
//!
//! The key is the user's own, supplied per call by the rules, and the only place it goes
//! is the `x-api-key` header of these requests.

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

/// How hard the model thinks. Below the model's default of `high`: the edits are a handful
/// of calls over a context it is given whole, the user is waiting on a phone, and a wrong
/// call is caught and sent back rather than applied.
const EFFORT: &str = "medium";

/// How often an answer the rules refused is sent back with the reason, after the first.
/// Most refusals are one call put right in one go, so a second retry is the margin.
const RETRIES: usize = 2;

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
inside a meal belongs to that meal, which every other day on that meal shares. For a meal \
that differs on some days (\"the same, but with less rice on Friday\"), create a second \
meal with those amounts and put that on those days.
- A date with no plan has nothing to add to or adjust. Give it one in the same reply: \
assign a plan that fits, or create a day plan holding what the request asks for and \
assign that.
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
only partly known because products do not state them. Be factual: no diagnosis and no \
supplement advice. Unless the user names days, use today and the days after it that have \
a plan.

Your text is shown in a chat on a phone as plain text — no Markdown, no tables, no \
headings; \"- \" for a list. Be brief: the user reads it at a glance. For a change, one \
short sentence saying what you did; the app shows the details itself, so do not repeat \
them. For a question, answer it directly in as few lines as it takes — the figures first, \
then at most one line on what is unknown. No preamble, no restating the question, no \
summary at the end, no caveats beyond that one line, no offers of further help. Use \
everyday words: no ids, no tool names. The app checks every operation \
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
        check: impl Fn(&[Operation]) -> Result<()> + Send + Sync,
    ) -> Result<Proposal> {
        let mut messages = conversation(history, request, context)?;
        let mut retries = 0;
        loop {
            let message = self.send(key, &body(&messages)).await?;
            let proposal = proposal(request, &message)?;
            let refusal = match check(&proposal.operations) {
                Ok(()) => return Ok(proposal),
                Err(error) => error.to_string(),
            };
            // Out of retries, the answer goes back as it is and the rules refuse it with
            // the same reason, which is then the user's to read.
            if retries == RETRIES {
                return Ok(proposal);
            }
            retries += 1;
            messages.push(json!({ "role": "assistant", "content": message.content }));
            messages.push(json!({ "role": "user", "content": refused(&message, &refusal) }));
        }
    }
}

impl Anthropic {
    /// One call to the Messages API.
    async fn send(&self, key: &str, body: &Value) -> Result<Message> {
        let url = format!("{}/v1/messages", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .header(header::ACCEPT, "application/json")
            .json(body)
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

        serde_json::from_str(&text).map_err(|source| Error::Internal {
            reason: format!("cannot read what the assistant answered: {source}"),
        })
    }
}

/// The request body: the system prompt, one tool per kind of operation, and the messages.
fn body(messages: &[Value]) -> Value {
    json!({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        // Adaptive is the only on-mode on this model, and resolving a sentence onto typed
        // edits over real ids is exactly the work that benefits from it.
        "thinking": { "type": "adaptive" },
        "output_config": { "effort": EFFORT },
        // The tools and the system prompt never change, so they are cached; the tools
        // come first in the prefix, so this one breakpoint covers both.
        "system": [{ "type": "text", "text": SYSTEM, "cache_control": { "type": "ephemeral" } }],
        "tools": tool::tools(),
        // Auto rather than forced: "none of these tools can do that" is a useful answer,
        // and a forced call would have to invent something.
        "tool_choice": { "type": "auto" },
        // And the messages up to the last one, so a retry reads the plan from the cache.
        "cache_control": { "type": "ephemeral" },
        "messages": messages,
    })
}

/// The messages of a first attempt: the conversation so far, and the user's correction
/// with the context it is about.
fn conversation(
    history: &[ChatTurn],
    request: &str,
    context: &AssistantContext,
) -> Result<Vec<Value>> {
    let context = serde_json::to_string(context).map_err(|source| Error::Internal {
        reason: format!("cannot describe the plan to the assistant: {source}"),
    })?;
    Ok(messages(
        history,
        &format!("<plan>\n{context}\n</plan>\n\n{request}"),
    ))
}

/// The answer to an attempt the rules refused: an error for each of its calls, the first
/// carrying the reason. The calls stand or fall together, so the model is asked for all
/// of them again rather than for the one that failed.
fn refused(message: &Message, reason: &str) -> Vec<Value> {
    message
        .tool_use_ids()
        .enumerate()
        .map(|(index, id)| {
            let content = if index == 0 {
                format!(
                    "The app refused these calls, so none of them was kept: {reason}. \
                     Make the whole set of calls again with that put right, or none if the \
                     request cannot be done. Reply to the user as if this were your first \
                     answer."
                )
            } else {
                "Not kept: the calls were refused together.".to_owned()
            };
            json!({ "type": "tool_result", "tool_use_id": id, "content": content, "is_error": true })
        })
        .collect()
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
fn proposal(request: &str, message: &Message) -> Result<Proposal> {
    if message.stop_reason.as_deref() == Some("refusal") {
        let detail = message
            .stop_details
            .as_ref()
            .and_then(|details| details.explanation.clone())
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
    for block in message.blocks() {
        match block {
            Block::Text { text } if !text.trim().is_empty() => prose.push(text.trim().to_owned()),
            Block::Text { .. } => {}
            Block::ToolUse { name, input, .. } => {
                operations.push(tool::operation(&name, input)?);
            }
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
    /// Kept as it came: a retry sends it back unchanged, thinking blocks and their
    /// signatures included, and reads it through [`Message::blocks`].
    #[serde(default)]
    content: Vec<Value>,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    stop_details: Option<StopDetails>,
}

impl Message {
    /// The blocks this crate reads; one it cannot is [`Block::Other`].
    fn blocks(&self) -> impl Iterator<Item = Block> + '_ {
        self.content
            .iter()
            .map(|block| Block::deserialize(block).unwrap_or(Block::Other))
    }

    fn tool_use_ids(&self) -> impl Iterator<Item = String> + '_ {
        self.blocks().filter_map(|block| match block {
            Block::ToolUse { id, .. } => Some(id),
            _ => None,
        })
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
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
    use std::sync::{Arc, Mutex};

    use mealprep_core::domain::{DateRange, Macros, Unit, parse_iso_date};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use uuid::uuid;

    use super::*;

    /// Recorded answers. No live call is made by this crate's tests: the request shape, the
    /// mapping back and the retries are what is under test, the last against a local
    /// stand-in for the API.
    const PROPOSAL: &str = include_str!("../fixtures/proposal.json");
    const CORRECTED: &str = include_str!("../fixtures/corrected.json");
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
        let body =
            body(&conversation(&[], "next 4 days I have 40 g less whey", &context()).unwrap());

        assert_eq!(body["model"], MODEL);
        assert_eq!(body["thinking"]["type"], "adaptive");
        assert_eq!(body["output_config"]["effort"], EFFORT);
        assert_eq!(body["tool_choice"]["type"], "auto");
        // The unchanging prefix is cached, and so is the plan for a retry.
        assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");
        assert_eq!(body["cache_control"]["type"], "ephemeral");
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
        let body = body(&conversation(&history, "make it 30 g instead", &context()).unwrap());
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
        let proposal = proposal("next 4 days I have 40 g less whey", &parse(PROPOSAL)).unwrap();

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
        let proposal = proposal("make it healthier", &parse(NOTHING_TO_DO)).unwrap();
        assert!(proposal.operations.is_empty());
        assert!(proposal.rationale.contains("which product"));
    }

    #[test]
    fn a_refusal_and_a_cut_off_answer_are_failures_rather_than_empty_proposals() {
        let Err(Error::Invalid { reason, .. }) = proposal("...", &parse(REFUSAL)) else {
            panic!("a refusal is not a proposal");
        };
        assert!(reason.contains("declined"), "{reason}");

        assert!(matches!(
            proposal("...", &parse(CUT_OFF)),
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

    /// A Messages API on a local port that answers each request with the next of
    /// `answers`, and the bodies it was sent.
    async fn messages_api(answers: Vec<&'static str>) -> (Anthropic, Arc<Mutex<Vec<Value>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let sent = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&sent);
        tokio::spawn(async move {
            for answer in answers {
                let (mut socket, _) = listener.accept().await.unwrap();
                let body = read_body(&mut socket).await;
                seen.lock()
                    .unwrap()
                    .push(serde_json::from_slice(&body).unwrap());
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{answer}",
                    answer.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
        (Anthropic::with_base_url(&base_url).unwrap(), sent)
    }

    /// The body of one request: the headers, then as much as they say there is.
    async fn read_body(socket: &mut tokio::net::TcpStream) -> Vec<u8> {
        let mut request = Vec::new();
        let mut chunk = [0; 8192];
        loop {
            let read = socket.read(&mut chunk).await.unwrap();
            request.extend_from_slice(&chunk[..read]);
            let Some(end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..end]);
            let length: usize = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().unwrap())
                })
                .unwrap();
            if request.len() >= end + 4 + length {
                return request[end + 4..end + 4 + length].to_vec();
            }
        }
    }

    /// The rules as the test stands them in: 40 g is more whey than any day has.
    fn refuse_forty_grams(operations: &[Operation]) -> Result<()> {
        let too_much = operations.iter().any(|operation| {
            matches!(
                operation,
                Operation::AdjustProductAmount { delta_amount, .. } if *delta_amount < -30.0
            )
        });
        if too_much {
            return Err(Error::Invalid {
                entity: Entity::AssistantRequest,
                reason: "2026-10-20 has 30 g of Whey, so 40 g cannot come off".to_owned(),
            });
        }
        Ok(())
    }

    #[tokio::test]
    async fn an_answer_the_rules_accept_is_asked_for_once() {
        let (assistant, sent) = messages_api(vec![CORRECTED]).await;
        let proposal = assistant
            .propose("key", &[], "20 g less whey", &context(), refuse_forty_grams)
            .await
            .unwrap();

        assert_eq!(proposal.operations.len(), 1);
        assert_eq!(sent.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_refused_answer_goes_back_with_the_reason_and_the_next_one_is_kept() {
        let (assistant, sent) = messages_api(vec![PROPOSAL, CORRECTED]).await;
        let proposal = assistant
            .propose("key", &[], "40 g less whey", &context(), refuse_forty_grams)
            .await
            .unwrap();

        // The second answer, prose and all; the first one's never reaches the user.
        assert_eq!(
            proposal.rationale,
            "Took 20 g off the scoop of whey on each of the next four days."
        );
        let [Operation::AdjustProductAmount { delta_amount, .. }] = proposal.operations[..] else {
            panic!("one adjustment: {:?}", proposal.operations);
        };
        assert_eq!(delta_amount, -20.0);

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 2);
        let messages = sent[1]["messages"].as_array().unwrap();
        // The first attempt, then the answer to it: the plan is not sent twice.
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0], sent[0]["messages"][0]);

        // The model's own turn goes back as it came, thinking block included.
        let first: Value = serde_json::from_str(PROPOSAL).unwrap();
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["content"], first["content"]);

        let results = messages[2]["content"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["type"], "tool_result");
        assert_eq!(results[0]["tool_use_id"], "toolu_01DemoAdjust");
        assert_eq!(results[0]["is_error"], true);
        let reason = results[0]["content"].as_str().unwrap();
        assert!(reason.contains("40 g cannot come off"), "{reason}");
    }

    #[tokio::test]
    async fn after_the_last_retry_the_answer_goes_back_for_the_rules_to_refuse() {
        let (assistant, sent) = messages_api(vec![PROPOSAL; RETRIES + 1]).await;
        let proposal = assistant
            .propose("key", &[], "40 g less whey", &context(), refuse_forty_grams)
            .await
            .unwrap();

        assert!(refuse_forty_grams(&proposal.operations).is_err());
        assert_eq!(sent.lock().unwrap().len(), RETRIES + 1);
    }
}
