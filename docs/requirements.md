# Software Requirements Specification — Mealprep

Structured after [IEEE Std 830-1998](http://www.cse.msu.edu/~cse870/IEEEXplore-SRS-template.pdf)
and the [markdown SRS template](https://github.com/antonmry/markdown-templates/blob/master/requirements.md).

This document supersedes the SRS of the Go/SvelteKit `mealprep` application. Requirement
identifiers are carried over unchanged wherever the requirement itself survived the port, so
`PR-8` means here what it meant there; identifiers retired by the port are listed in
Appendix D rather than reused.

## 1. Introduction

### 1.1 Purpose
Mealprep helps one person plan a week of meals against daily macro targets — calories, fat,
protein and carbohydrates — and see what the plan costs.

### 1.2 Scope
Mealprep is a single application that runs entirely on the user's device: a webview frontend,
the business rules and the database in one process, with no server and no network service.
It covers configuring products, composing meals from them, grouping meals and products into
day plans, assigning day plans to calendar days, recording body weight, reporting nutrients
and cost against targets, and correcting a plan by describing the change in plain language.
It does not cover shopping lists, recipes with instructions, multiple users or sharing.

Cost tracking was excluded by the predecessor and is now in scope (§4.6). Categories for
meals and day plans are new (§4.5), as is weight tracking (§4.10) and the planning
assistant (§4.12). The assistant is the first feature that sends any of the user's data off
the device, and it does so only with a key the user supplies; §5.3 says what is sent.

### 1.3 Document conventions
Requirements are identified as `XX-n` (`PR-1`, `ML-2`) and are binding. Priority is stated
per feature, not per requirement. *To be determined* in italics marks an open item; every
one is listed with its blocking condition in Appendix C.

### 1.4 Intended audience
The developers building and maintaining this repository. Read Section 2 for context,
Section 4 for what to build, Sections 3 and 5 for the constraints around it.

### 1.5 Definitions and abbreviations
- **Product** — a food item with its nutrients per 100 g, or per 100 ml when it is a liquid. Imported from Open Food Facts or created by the user.
- **Meal** — a labeled set of products, each with a serving size in grams or millilitres.
- **DayPlan** — a labeled, ordered list of items, where an item is a whole meal or a product eaten on its own.
- **Category** — an optional grouping a meal or a day plan belongs to. A category belongs to one scope and cannot move.
- **CalendarDay** — a date with a day plan assigned to it.
- **Targets** — the user's daily figures for energy (kcal), fat, protein and carbohydrates.
- **Macro** — one of those four figures.
- **Cost** — what a composition costs, derived from its products' prices, carrying whether every product in it had one.
- **Weigh-in** — one body weight recorded against a date, in kilograms. A date holds at most one.
- **Trend** — the exponentially smoothed average of the weigh-ins, which is the figure to read rather than the last measurement.
- **OFF** — [Open Food Facts](https://world.openfoodfacts.org/), the external product database.
- **Catalogue** — OFF as the application sees it, behind the `Catalogue` trait.
- **Assistant** — what turns a plain-language request into typed operations, behind the `Assistant` trait. Claude, reached over HTTPS with the user's own API key.
- **Operation** — one typed edit the assistant may propose, bounded to the dates it acts on. The set is closed, and the application rather than the model decides what each one means.
- **Proposal** — what the assistant answered: operations and its prose, which is the whole answer when the request was a question. Unchecked, and never applied without a confirmation.
- **Preview** — a checked proposal: which dates would change, what changes on each, and where that leaves the day against the targets.
- **Day-local copy** — one date's own copy of a day plan, made so that editing that date cannot move every other date assigned the same plan. It carries `source_plan_id`, and appears in no list and no picker.
- **SRS** — this document.

### 1.6 References
- [Open Food Facts](https://world.openfoodfacts.org/) — external product database.
- `docs/architecture.md` — system structure and key decisions.
- `docs/code-standards.md` — binding implementation constraints.
- `docs/todo.md` — deferred decisions, not to be implemented unless asked.

## 2. Overall description

### 2.1 Product perspective
The successor to the Go/SvelteKit `mealprep` application, which ran a browser frontend
against a self-hosted HTTP backend. This version collapses that into one process on the
device: the webview invokes Tauri commands directly, and the HTTP API and its OpenAPI
contract are gone. The domain rules, the entities and the product catalogue behaviour are
carried over. See `docs/architecture.md` for the component diagram.

### 2.2 Product functions
- Configure products, by importing from OFF — by name search or by scanning a barcode — or creating them by hand.
- Compose meals from products with serving sizes.
- Group meals and loose products into day plans, in one order.
- Organise meals and day plans into categories.
- Assign day plans to calendar days.
- Set daily macro targets and see planned nutrients and cost against them.
- Record body weight and read its smoothed trend and rate of change.
- Correct the plan by describing the change — "next 4 days I have 40 g less whey" — and confirming the edit the assistant proposes.

### 2.3 User classes and characteristics
One class, the single user, who both configures products and plans meals. No roles, no
administrator, no authentication. The user is assumed to understand macros well enough to
set their own targets.

### 2.4 Operating environment
- Application: Rust (edition 2024, `rust-version` 1.94) in a Tauri v2 shell. One process, no server.
- Storage: a SQLite database in the platform's app data directory, reached through `sqlx`.
- Frontend: SvelteKit + TypeScript (Svelte 5 runes), built with Bun into a static single-page app that Tauri serves into its webview.
- Android is the primary target; the desktop build runs the same code. The page only works inside the Tauri webview and says so when opened in a plain browser.

### 2.5 Design and implementation constraints
- `docs/code-standards.md` governs all implementation and overrides nothing in this document.
- The command surface in `src-tauri/src/command/` is the only path from the frontend to the rules.
- Persistence sits behind the `core::store` traits so the database stays swappable.
- `mealprep-core` performs no I/O and must not learn about SQLite, HTTP or Tauri.
- No authentication and no user accounts while the product is single-user and on-device.

### 2.6 User documentation
*To be determined* (TBD-3).

### 2.7 Assumptions and dependencies
- The OFF API is reachable and free to use under its terms; the application degrades to hand-created products when it is not.
- An OFF entry missing the four macros seeds the import form empty rather than zero, and the user completes it (`IM-4`).
- One user, one device, one dataset. Nothing synchronises.

## 3. External interface requirements

### 3.1 User interfaces
A webview UI built from plain CSS with no component library, laid out mobile-first around a
persistent bottom tab bar. The predecessor's sidebar, command palette and keyboard shortcuts
are gone: the primary device is a phone, where neither a sidebar nor `⌘K` has a place.

Nutrients are shown as the four macros, with the full nutrient set behind an expansion
(`PR-9`), ordered as Regulation (EU) 1169/2011 Annex XV presents a nutrition declaration.

| Screen | Path | Serves |
| --- | --- | --- |
| Product list | `/products` | `PR-10`, `PR-11`, `CO-3` — filter by name or brand, sort by name or any macro |
| New product | `/products/new` | `PR-3`, `PR-7`, `CO-1`, `PL-1` |
| Catalogue search | `/products/search` | `PR-1`, `PR-2`, `PR-8`, `SC-3` |
| Import review | `/products/import/[code]` | `IM-1`–`IM-4`, `SC-2` — where a scan and a search result both land |
| Product detail | `/products/[productId]` | `PR-9`, `CO-3`, §6 — macros, nutrient expansion, source credit and link |
| Edit product | `/products/[productId]/edit` | `PR-10`, `PL-1` |
| Meal list | `/meals` | `ML-5`, `CT-3` |
| Meal categories | `/meals/categories` | `CT-1`, `CT-2`, `CT-5` |
| New meal | `/meals/new` | `ML-1`, `ML-2`, `ML-6`, `CT-3` |
| Meal detail | `/meals/[mealId]` | `ML-3`, `ML-4`, `CO-2` |
| Edit meal | `/meals/[mealId]/edit` | `ML-5`, `ML-6` |
| Day plan list | `/day-plans` | `DP-5`, `TG-2`, `CT-3` |
| Day plan categories | `/day-plans/categories` | `CT-1`, `CT-2`, `CT-5` |
| New day plan | `/day-plans/new` | `DP-1`, `DP-2`, `DP-6` |
| Day plan detail | `/day-plans/[dayPlanId]` | `DP-4`, `TG-2`, `TG-3`, `CO-2` |
| Edit day plan | `/day-plans/[dayPlanId]/edit` | `DP-5`, `DP-6` |
| Calendar | `/calendar` | `CL-1`–`CL-5`, `TG-2`, `TG-3` |
| Targets | `/targets` | `TG-1`, `WT-6`, `DT-1`–`DT-4` — the macro targets, the weight trend beside them, and exporting or importing the data |
| Weight | `/weight` | `WT-1`–`WT-7` — quick log, the graph, and the weigh-in history |
| Assistant | `/assistant` | `AS-1`–`AS-17` — the whole screen is the chat: each request with its preview and Apply or Discard, or the answer to a question; the API key field only while none is stored |

Creating and editing are screens rather than dialogs, because every write replaces the whole
entity and so has to start from a fresh read. Dialogs carry what is genuinely one decision:
assigning a plan to a date, applying a plan across weekdays, managing categories, and
confirming a delete.

**The composition editors** — meals and day plans — share one layout: identity, the ordered
rows, and the derived total shown as not editable (`ML-3`, `DP-4`). Rows are added by picking
the item, which creates the row already populated, and are reordered with move-up and
move-down buttons. Reordering is never drag-only: WCAG 2.2 SC 2.5.7 requires a single-pointer
alternative to any dragging motion, so buttons are the baseline rather than the fallback.

**The calendar** shows four weeks at a time and pages by the whole block (`CL-5`). An
unplanned day reads as unplanned and stays out of every denominator, rather than reading as a
shortfall — the target is a daily figure, so a four-week total cannot be read against it
directly.

**Planned against target** is drawn as a meter per macro, never a ring: a ring encodes
0–100% and has no honest way to render 130%, which `TG-3` requires.

**Colour.** Protein, fat and carbs each carry a fixed hue in every view — Okabe-Ito orange,
reddish purple and bluish green — held apart far enough to survive the common colour-vision
deficiencies. Energy stays neutral, because it is the sum of the other three rather than a
fourth sibling. No macro is red, and no macro uses the interactive accent: red in this
application always means something is broken, never that a target was exceeded (`TG-3`).
Colour is never the only channel — every figure is also labelled, and the series order never
changes.

### 3.2 Hardware interfaces
The device camera, on phones only, for barcode scanning (`SC-1`). Nothing else.

### 3.3 Software interfaces
- **Open Food Facts REST API** — read-only, over rustls with webpki roots. Search by text and fetch by barcode, behind the `Catalogue` trait.
- **Anthropic Messages API** — one request per question the user asks, and up to two more when the application would refuse the answer (`AS-17`), over rustls with webpki roots, authenticated with the key the user pasted in, behind the `Assistant` trait. One tool per kind of operation, so the answer is tool calls rather than prose. §5.3 says what the request carries.
- **SQLite** — a local file, reached only through `mealprep-sqlite`.
- **Barcode scanner** — `tauri-plugin-barcode-scanner`, on Android and iOS only.

### 3.4 Communications interfaces
Between the frontend and the rules there is no network protocol: the webview invokes Tauri
commands over the IPC bridge, and arguments and results are JSON-serialised domain types.
The outbound traffic is HTTPS to the OFF API, and — only while the user has stored an API
key, and only when they ask a question — HTTPS to the Anthropic Messages API. The
application listens on no port.

## 4. System features
Features are ordered by the dependency between their domains: products come first, meals
build on products, day plans build on both.

### 4.1 Products
**Description and priority.** The unit the user configures first. High.

**Stimulus/response.** The user searches OFF by name or scans a barcode, reviews what the
catalogue supplied, and a snapshot of that product joins their own set. Or the user fills in a
product by hand and it joins the same set.

**Functional requirements.**
- `PR-1` The system shall let the user search the OFF database by text and show the matches.
- `PR-2` The system shall let the user add a searched product to their own products.
- `PR-3` The system shall let the user create a product by entering its name and nutrients per 100 g, or per 100 ml when it is a liquid.
- `PR-4` A product created by the user shall carry the same fields as an imported one.
- `PR-5` The system shall treat imported and hand-created products identically everywhere downstream; nothing shall depend on a product's origin.
- `PR-6` The system shall store every nutrient value an imported product carries, not only the four macros.
- `PR-7` The system shall require the four macros when the user creates a product by hand, and shall accept any further nutrient as optional.
- `PR-8` The system shall snapshot an imported product's data at pick time and shall not re-read it from OFF afterwards.
- `PR-9` The system shall display the four macros, and shall offer the remaining nutrient information behind an expansion.
- `PR-10` The system shall support create, read, update and delete for products, where delete is soft.
- `PR-11` The system shall record a product's brand where the catalogue carries one, and shall treat it as part of the product's identity rather than as presentation. It is optional for a product created by hand.

### 4.2 Meals
**Description and priority.** A labeled set of products with serving sizes. High.

**Functional requirements.**
- `ML-1` The system shall let the user create a meal with a label.
- `ML-2` The system shall let the user add products to a meal, each with a serving size in the product's unit.
- `ML-3` The system shall store a meal's composition only, and derive its nutrients on read.
- `ML-4` The system shall show a meal's computed nutrients alongside it.
- `ML-5` The system shall support create, read, update and delete for meals, where delete is soft.
- `ML-6` The system shall let the user order a meal's servings, and shall preserve that order.

### 4.3 Day plans
**Description and priority.** A labeled, ordered list of what is eaten in a day. High.

**Functional requirements.**
- `DP-1` The system shall let the user create a day plan with a label.
- `DP-2` The system shall let the user add whole meals and individual products to a day plan, in one shared order, because a day is eaten in one sequence.
- `DP-3` A day plan shall carry no properties beyond its label, its optional category and its items.
- `DP-4` The system shall show a day plan's nutrients as the sum of its items'.
- `DP-5` The system shall support create, read, update and delete for day plans, where delete is soft.
- `DP-6` The system shall let the user order a day plan's items, and shall preserve that order. A day plan has no time-of-day slots, so the order is the only sequence it carries.
- `DP-7` A product whose row is deleted shall remain in a day plan as a removed entry keeping its id, while a deleted meal shall drop out of it.

### 4.4 Calendar
**Description and priority.** Assignment of day plans to dates. Medium.

**Functional requirements.**
- `CL-1` The system shall let the user assign a day plan to a calendar day.
- `CL-2` The system shall let the user view the assigned plans of the weeks in view.
- `CL-3` A calendar day shall hold at most one day plan. Assigning a plan to a day that already has one shall replace it.
- `CL-4` The system shall let the user assign one day plan to several days at once, naming the days it will touch before writing anything.
- `CL-5` The system shall show four consecutive weeks at a time, and shall page forward and back by four weeks.

### 4.5 Categories
**Description and priority.** Optional grouping for meals and day plans. Medium. New in this
version.

**Functional requirements.**
- `CT-1` The system shall let the user create, rename and delete categories.
- `CT-2` A category shall belong to one scope — meals or day plans — fixed at creation and never changed afterwards.
- `CT-3` A meal or a day plan shall belong to at most one category, and may belong to none.
- `CT-4` A label shall be unique within its category, ignoring case; entities without a category form one scope of their own for this purpose.
- `CT-5` Deleting a category shall not delete what it holds; those entities become uncategorised.

### 4.6 Cost
**Description and priority.** What a plan costs. Medium. Out of scope in the predecessor.

**Functional requirements.**
- `CO-1` The system shall let the user record what 100 units of a product cost, as an optional figure.
- `CO-2` The system shall derive the cost of meals and day plans from their products' prices, the way macros are derived, and shall never store it.
- `CO-3` A derived cost shall carry whether every product in the composition had a price. Where one did not, the figure shall be presented as a floor rather than a total, so an unpriced product is never silently counted as free.

### 4.7 Catalogue import
**Description and priority.** Reviewing what a crowd-sourced entry supplies before keeping it.
Medium. New in this version.

**Functional requirements.**
- `IM-1` The system shall show an imported entry for review before it is stored, and shall store only what the user confirmed.
- `IM-2` The review shall name who supplied the entry, distinguishing a manufacturer-supplied entry from a crowd-sourced one.
- `IM-3` The review shall surface the catalogue's own figure-related quality warnings about the entry, and when it was last modified.
- `IM-4` A macro the entry does not carry shall seed the form empty rather than zero.
- `IM-5` The system shall record the catalogue code an imported product came from, as its provenance.

### 4.8 Barcode scanning
**Description and priority.** Getting to an entry without typing. Medium. New in this version.

**Functional requirements.**
- `SC-1` The system shall let the user scan a product barcode with the device camera on platforms that have one.
- `SC-2` A successful scan shall open the import review for that code.
- `SC-3` On a platform without a camera, a barcode typed into catalogue search shall reach the same review.
- `SC-4` The scan shall be cancellable, including by back navigation, and shall supply its own frame and cancel control because the plugin draws none.

### 4.9 Targets and feedback
**Description and priority.** The comparison the product exists for. Medium.

**Functional requirements.**
- `TG-1` The system shall let the user set daily targets for calories, fat, protein and carbohydrates.
- `TG-2` The system shall show the four planned macros next to the targets.
- `TG-3` The system shall display deviations from the targets and shall never block or alter a plan because of them.
- `TG-4` Energy shall be expressed in kcal, mass in grams and volume in millilitres, throughout the product.

### 4.10 Weight
**Description and priority.** What the targets exist to move. Medium. New in this version.

**Stimulus/response.** The user records the weight the scale showed. The application smooths
it into a trend, draws it against the previous weigh-ins, and says which way it is going.

**Functional requirements.**
- `WT-1` The system shall let the user record a body weight in kilograms against a date, defaulting to today.
- `WT-2` A date shall hold at most one weigh-in. Recording again for a date shall replace it, and the user shall be told that it will before they save.
- `WT-3` The system shall support reading and deleting weigh-ins, where delete is soft.
- `WT-4` The system shall derive a trend from the weigh-ins by exponential smoothing, and shall treat the days between two weigh-ins as though the weight moved evenly across them. A trend shall never be stored.
- `WT-5` The system shall present the trend as the headline figure rather than the most recent measurement, and shall draw the measurements alongside it rather than instead of it.
- `WT-6` The system shall derive a rate of change in kilograms per week from the trend, and shall withhold it until the history is long enough for the figure to mean anything.
- `WT-7` The graph shall scale to the data rather than to zero, and shall leave enough room below the lowest value that ordinary fluctuation cannot read as collapse.
- `WT-8` The system shall refuse a figure outside the plausible human range, and shall otherwise have no opinion about the weight recorded — no goal, no judgement, no congratulation.

### 4.11 Plausibility checks
**Description and priority.** Catching a typo without refusing the save. Low. New in this
version.

**Functional requirements.**
- `PL-1` The system shall check a product's figures against each other as they are entered — energy against the macros, kJ against kcal, saturates against fat, sugars against carbohydrates, and the total against 100 g.
- `PL-2` A failed check shall warn and shall never block a save, consistent with `TG-3`.

### 4.12 Planning assistant
**Description and priority.** Correcting a plan by describing the change. Medium. New in
this version.

**Stimulus/response.** The screen is a chat. The user writes what changed — "next 4 days I
have 40 g less whey". The application sends that, the earlier messages of the chat and a
compact view of the plan to the assistant, which answers with typed operations. The
application checks every one against the stored data and shows what would change on each
date, as the reply. The user applies it, discards it, or replies — "make it 30 g instead" —
and gets a new proposal in its place.

**Functional requirements.**
- `AS-1` The system shall let the user describe a correction to their planning in plain language.
- `AS-2` The system shall turn a request into operations drawn from a closed, typed set, and the assistant shall never write to the database. It proposes; the application applies.
- `AS-3` The system shall check every proposed operation against the stored data — the product, meal or plan it names, the amount it would leave, the dates it covers — and shall refuse the request rather than silently adjust it.
- `AS-4` The system shall show a preview before anything is written: which dates would change, what changes on each, and the resulting macros against the targets, consistent with `TG-3`.
- `AS-5` The system shall write nothing without an explicit confirmation, shall offer discarding as an equal choice, and shall check the confirmed operations again before applying them.
- `AS-6` An operation bounded to dates shall change only those dates. Each shall be detached onto its own copy of its assigned day plan before being edited, so no other date assigned the same plan moves; a day-local copy shall not appear in the day plan list or in any picker.
- `AS-7` The system shall refuse to change a date before today unless the request explicitly and unambiguously names a past day.
- `AS-8` The system shall record every applied request — what was asked, what the assistant said, the operations, what each date pointed at before, and when. The record is kept in the database and its export rather than shown on screen, and shall be enough to reverse the change by hand.
- `AS-9` The system shall require the user's own API key for the assistant, shall keep it on the device, shall never return it in full, and shall send nothing to the assistant while there is none.
- `AS-10` Adding a kind of operation shall not change how a proposal is previewed, confirmed or applied.
- `AS-11` The system shall let the assistant propose creating a product, meal or day plan, and changing any field of an existing one or of the targets — a product's name, brand, unit, macros, further nutrients and price, a meal's or plan's name, category and contents — each checked by the same rules the screens' own forms apply. A change to an existing one shall touch only the fields it names, and shall be able to remove a field a product may lack.
- `AS-12` The preview shall name each product, meal or day plan a proposal creates or changes and say what changes in plain words; for a change it shall say how many dates in view use it, because they all follow. It shall show no identifiers and no operation names.
- `AS-13` A proposal may use what it creates — a meal made of a new product, a new plan put on a date — and applying it shall write those first and point everything after them at the identifiers the store assigns.
- `AS-14` The system shall let the assistant propose syncing a product from Open Food Facts, by the code it was imported under or a barcode the user gives. The application, not the model, shall fetch the entry, both when previewing and again when applying; the product shall keep its name and price, keep any of the four macros the entry lacks, and be linked to the code it was synced from. The preview shall say what each figure changes from and to, and pass on the catalogue's own warnings.
- `AS-15` The system shall let the user ask a question about their plan — such as how its micronutrients compare with the reference intakes — and shall show the assistant's answer with nothing to apply. The answer shall compare against the EU reference intakes for adults unless the user gives other figures, say which nutrients the products leave unknown, and give no diagnosis or supplement advice.
- `AS-16` The system shall present the assistant as a chat: each request and its reply in order, with the earlier requests and replies of the chat sent with each new one so a request may refer back to them, and with what became of each proposal — applied or discarded — shown on it and told to the assistant. Only the latest proposal can be applied; sending another message discards one still waiting. The chat is held in memory only, can be started over, and checks each request against the stored data on its own words alone (`AS-3`, `AS-7`). The whole screen shall be the chat, with no canned requests, and replies shall be brief: one sentence for a change, whose details the preview shows, and the figures first for a question.
- `AS-17` The system shall, when it would refuse what the assistant proposed, give the assistant the reason and ask it again, at most twice, before showing the user anything, so that a request needing several edits is not lost to one wrong one. Only the final answer is shown, and it is checked from scratch like any other (`AS-3`).

### 4.13 Data export and import
**Description and priority.** Moving all of the user's data in and out of the application as
one file. High: the database is otherwise a private file on the phone, so an uninstall or a
second install is a loss with no way back. New in this version.

**Stimulus/response.** Export asks where to save, through the system's file picker, and
writes a copy of everything there. Import asks for a file the same way, checks it, keeps a
copy of what it replaces, swaps it in and closes the application.

**Functional requirements.**
- `DT-1` The system shall export the whole database as one self-contained file, consistent at the moment it is taken, without closing the application.
- `DT-2` The system shall check a file before importing it — an intact SQLite file whose applied migrations are all ones this version carries — and shall refuse it, replacing nothing, otherwise. A file from an older version shall be accepted and migrated as an update would be.
- `DT-3` The system shall keep a copy of the current data before an import replaces it, and shall ask for confirmation first.
- `DT-4` The system shall save and read the file wherever the user chooses through the platform's file picker, and shall need no storage permission to do it.

## 5. Other non-functional requirements

### 5.1 Performance requirements
Out of scope as a target, with one constraint from the platform: the database is on a phone
and every read crosses the IPC bridge, so a screen loads what it displays and nothing more.

### 5.2 Safety requirements
None. The application gives no medical or dietary advice and enforces nothing, per `TG-3`
and `PL-2`.

### 5.3 Security requirements
The application has no authentication and no transport security of its own: it listens on no
port and accepts no connection, so there is nothing to authenticate. Its database is a
private file in the platform's app data directory, and nothing in the application copies it
anywhere but where the user exports it (§4.13).

Two features reach outward, both as HTTPS clients over rustls with webpki roots, and both
only when the user asks.

**Open Food Facts (§4.1, §4.7).** A search term or a barcode goes out and a catalogue entry
comes back. None of the user's own data is sent.

**The planning assistant (§4.12).** This is the one feature that sends the user's own data
off the device. It is off until the user turns it on by supplying an API key.

- *What is sent, per message:* the request the user typed, the earlier requests of the chat
  with the assistant's replies to them and whether each proposal was applied (at most the
  last twenty, and only the words: no operations, no ids), and a view of the plan
  around today — the dates in the assistant's window with the labels and amounts on them and
  their nutrient totals, their day plans with what they hold, their meals with their
  servings, their products with brand, macros and further nutrients per 100 g or ml and
  whether they came from the catalogue, the categories, all with their ids, and the macro
  targets.
- *What is sent again when an answer is refused (`AS-17`):* the same messages, with the
  assistant's answer and the application's reason for refusing it. Nothing more of the
  plan goes with it.
- *What a sync sends elsewhere:* the product's code goes to Open Food Facts, as a barcode
  lookup does; nothing of the user's own goes with it.
- *What is not sent:* the database, the weigh-ins, the prices, the record of applied
  requests, any identifier of the device or the installation, and anything about dates
  outside that window.
- *Where the key lives:* one row of `app_settings` in the same local database, stored as the
  user pasted it, and read only to be put in the `x-api-key` header of those requests. No
  command returns it in full: the frontend is given a flag and the last four characters, so
  the user can tell which key is stored without the application handing it back.
- *How it is switched on and off:* by storing the key. The screen offers no way to replace or forget it once stored; clearing the app's data does. With no key
  stored the assistant sends nothing at all — the application does not reach the model to
  find out whether it would have worked. The screen says what will leave the device before
  the key is entered rather than after.
- *What is trusted:* nothing the model answers. It proposes typed operations, which are
  checked against the stored data and applied only on the user's confirmation (`AS-2`,
  `AS-3`, `AS-5`), so a wrong or hostile answer cannot write anything the user has not read
  first.

What happens to the data at the other end is governed by Anthropic's terms, not by this
document. Sending plan context to a third party is a trade the user makes per installation,
by choosing to store a key.

Giving this application a network service, synchronising its database anywhere, or sending
any further part of it outward invalidates this section and requires it to be rewritten
first.

### 5.4 Software quality attributes
- **Maintainability** — `docs/code-standards.md` is binding; the tree stays formatted and lint-clean.
- **Portability** — persistence is reachable only through the `core::store` traits, so the database can be swapped by adding a crate that implements them.
- **Correctness** — the rules live in `mealprep-core`, which has no I/O and is tested without one.
- Reliability and availability targets: out of scope, as in 5.1.

### 5.5 Logical database requirements
The database stores products with their full nutrient set and optional price, categories,
meals with their servings, day plans with their ordered items — the day-local copies the
assistant makes included, marked by `source_plan_id` — calendar assignments, weigh-ins,
targets, the applied assistant requests, and the settings the user supplies, of which the
assistant's API key is the only one. Nutrient and cost totals are never stored; they are derived on read. Every table
carries `created_at`, `updated_at` and `deleted_at`; a delete sets `deleted_at` and removes no
row, and reads exclude soft-deleted rows. The schema is `crates/sqlite/migrations/`, applied
on open — which closes the predecessor's TBD-12 and its hand-run `ALTER TABLE`s.

### 5.6 Business rules
- The application informs and never enforces (`TG-3`, `PL-2`).
- A product's origin never changes its behaviour (`PR-5`).
- Nothing is deleted outright; entities are soft-deleted (5.5).
- A derived figure never hides what it is missing (`CO-3`).
- The trend is the figure worth reading; a single weigh-in is mostly noise (`WT-5`).
- The assistant proposes and the application applies; nothing a model answers is written unreviewed (`AS-2`, `AS-5`).
- An edit bounded to dates never moves a date it did not name, and never moves the past (`AS-6`, `AS-7`).

## 6. Other requirements
Open Food Facts data is published under the Open Database License (ODbL), with its individual
contents under the Database Contents License. The application shall credit Open Food Facts
wherever imported product data is shown, and shall link to the source entry on a product's
detail view. ODbL share-alike obligations attach to redistributing the database; a single
on-device instance redistributes nothing, so nothing further is required today. Publishing
this application's data, or shipping it with a bundled catalogue, revisits this section.

## Appendix A: Glossary
See 1.5.

## Appendix B: Analysis models
`docs/architecture.md` holds the component diagram, the crate layout and the domain model.

## Appendix C: To be determined list
IEEE 830 §4.3.3.1 asks that every TBD record why it is open and what closes it.

| ID | Open item | Why it is open | What closes it |
| --- | --- | --- | --- |
| TBD-3 | User documentation | Premature while the app is used by the person building it | Someone else installs it |
| TBD-13 | Whether a day plan's cost should be shown per day or per plan | Only the plan-level figure exists, and no view has needed the other | A view needs a per-day cost |
| TBD-14 | Whether weight should be read against energy intake | The calendar knows what was planned per day, and the trend knows what happened; nothing joins them | Wanting to see the trend against the calories that produced it |

The predecessor's TBD-12 (database schema) is closed: the schema is
`crates/sqlite/migrations/0001_initial.sql`, applied by `sqlx::migrate!` on open.

## Appendix D: Requirements retired by the port
Identifiers that were binding in the predecessor and are not reused here.

| ID | Was | Why it is gone |
| --- | --- | --- |
| `PR-7` (second clause) | An OFF entry missing the four macros shall be rejected at import | Replaced by `IM-4`: the entry is shown for review with those macros empty, and the user completes them. Rejecting a usable entry over a missing figure cost more than it saved. |
