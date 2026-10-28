# Todo

Deferred decisions. Nothing here is implemented unless it is explicitly asked for. Finding an
entry relevant to the task at hand is not permission to act on it: the current behaviour
documented in `docs/requirements.md` stands until the entry is picked up deliberately.

Each entry records what happens today, what is deferred, and what would trigger the change.

## Frontend test runner
- **Today**: `bun test src` covers the pure modules under `src/lib/domain` — macro
  arithmetic, week dates, the calendar join, plausibility, sorting — with no config and no
  extra dependency.
- **Deferred**: Vitest, which resolves `$lib` and `$app` and can render components.
- **Trigger**: the first unit test that needs to import `$app/*` or mount a component.

## End-to-end tests
- **Today**: none. The predecessor ran Playwright against a live backend; there is no backend
  to run against, and driving the app means driving a Tauri window or an Android emulator.
- **Deferred**: WebDriver through `tauri-driver` on desktop, or an emulator-based run.
- **Trigger**: a regression that a `lib/domain` test could not have caught, or a release
  process that needs a smoke test before shipping an APK.

## Frontend formatter
- **Today**: none. The predecessor used Biome; nothing formats or lints the frontend here, and
  the standard is to match the file being edited.
- **Deferred**: adopting a formatter for `.svelte` and `.ts`.
- **Trigger**: formatting churn that costs review time, or a second person editing the
  frontend.

## Switching the store at runtime
- **Today**: `AppState` in `src-tauri/src/setup.rs` is a type alias, so the store is chosen at
  compile time and dispatched statically.
- **Deferred**: an enum store or boxed traits, so a build can carry both a local and a remote
  store and pick between them.
- **Trigger**: a second store existing at all.

## A cloud database and sync
- **Today**: one SQLite file on the device, and nothing synchronises. The security section of
  the SRS rests on the database never leaving: the assistant sends a view of the plan when
  asked, which is a question about the data rather than a copy of it.
- **Deferred**: a crate implementing the `core::store` traits against Postgres, and a sync
  design for reconciling two stores.
- **Trigger**: wanting the same data on a second device.

## iOS
- **Today**: the barcode scanner is compiled for Android and iOS, and `src-tauri/gen` holds
  an Android project only. Nothing has been built or run for iOS.
- **Deferred**: `tauri ios init` and whatever the build needs.
- **Trigger**: access to a Mac and a reason to ship there.

## The unused radar and heatmap modules
- **Today**: `src/lib/domain/radar.ts` and `src/lib/domain/heatmap.ts` are ported from the
  predecessor, fully tested, and imported by nothing. The features they served — the day plan
  radar and the shaded macro columns — were dropped from the UI in the port.
- **Deferred**: either drawing them in the new mobile layout, or deleting both modules and
  their tests.
- **Trigger**: a screen that wants either visualisation, or a cleanup pass that would rather
  not carry dead code.

## A goal weight
- **Today**: weight is recorded and trended, with no goal and no opinion about the figure.
  The rate of change says which way it is going, which is what there is to act on.
- **Deferred**: storing a target weight and drawing it as a line on the chart. It fits the
  Targets concept, and it is one field, one command and one line on the plot.
- **Trigger**: wanting to see the distance to a goal rather than the direction of travel.
  Deliberately excluded from "simple" — see `docs/research/2026-09-20-weight-tracker-ui-ux.md`.

## Weight against energy intake
- **Today**: the calendar knows what was planned per day and the weight trend knows what
  happened, and nothing joins them.
- **Deferred**: a chart of the trend against the calorie balance that produced it, which is
  the question a food planner with a weight tracker in it can uniquely answer.
- **Trigger**: wanting to know whether the plan is working, rather than what the plan is.
  Tracked as TBD-14 in the SRS.

## Per-day cost on the calendar
- **Today**: cost is derived for meals and day plans. The calendar shows macros per day and
  per week, and no cost figure.
- **Deferred**: whether a day's or a week's cost is worth showing, and whether an incomplete
  cost can be summed across days without becoming meaningless.
- **Trigger**: wanting to read a week's spend off the calendar. Tracked as TBD-13 in the SRS.

## Getting the data off the device
- **Today**: the database is a private file in the app's Android storage with nothing that
  exports it. `mise run android:db:pull` copies it out, but only from a debug install, so a
  release-only device has no path to its own data. An uninstall — which a change of signing
  key forces — takes the database with it, as one did on 2026-09-20.
- **Deferred**: an in-app export, which would be the first feature to admit a second copy of
  the dataset and sits against `docs/requirements.md` "One user, one device, one dataset.
  Nothing synchronises."
- **Trigger**: wanting the data to survive the device, or a second device at all.

## An assistant that runs on the device
- **Today**: the assistant is `crates/anthropic`, which sends the request and a compact view
  of the plan to Claude Sonnet over HTTPS with the user's own API key. That is the one thing
  in the app that sends the user's data anywhere, and `docs/requirements.md` §5.3 says what.
- **Deferred**: a crate implementing `core::store::Assistant` against a model small enough to
  run on the phone, which would take the network — and the key, and §5.3 — back out. It was
  judged and rejected rather than overlooked: resolving one sentence onto several typed edits
  over real ids and amounts, all of them right or refused, is not what a phone-sized model is
  reliable at, and the failure mode is a plausible edit naming the wrong product.
- **Trigger**: a local model that holds up on this task, or a user who will not supply a key.
  `core` does not know Anthropic exists, so it is one crate and one line of `setup.rs`.

## Where the assistant's API key is stored
- **Today**: one row of `app_settings` in the application's own database, as the user pasted
  it. The file is private to the application on Android, no command returns the key in full,
  and `docs/requirements.md` §5.3 says so plainly. A debug install is still readable with
  `mise run android:db:pull`, which is how the database is meant to be reachable.
- **Deferred**: the Android Keystore, through `EncryptedSharedPreferences` or a Tauri plugin
  over it, so the key is not in the database at all.
- **Trigger**: a second secret worth protecting, a key that is not the user's own to lose,
  or shipping to anyone but this user.

## Undoing an applied assistant request
- **Today**: every applied request is recorded with the operations and, per date, what that
  date pointed at before — by label and by id. That is enough to reverse a change by hand:
  assign the previous plan to the date again, or clear the date when it had none. The screen
  shows that line beside each change.
- **Deferred**: an undo command that does it in one tap. The record already holds everything
  it needs; what is missing is the decision about what undoing a request that was partly
  overwritten since should do, and what to do with the day-local copies it leaves behind.
- **Trigger**: reversing one by hand often enough to want the button, or a request that goes
  wrong in a way the log reads clearly but is tedious to put back.

## Naming one of two identical items in a day
- **Today**: an assistant operation addresses an item by id, and a day can hold the same
  product or meal twice — whey loose at breakfast and again after training. `sole_position`
  in `service::assistant` refuses such a request as `Invalid` and says the item is on that
  date more than once, rather than editing the first entry and showing a change line that
  does not say which one moved.
- **Deferred**: addressing an item by its position in the day's order as well as by id, so
  "the one after training" can be expressed. It is a field on three operations and a tool
  schema change, and it makes every proposal carry a number the user cannot see.
- **Trigger**: a day worth planning that genuinely holds one product twice, often enough
  that editing the plan by hand is the slower path.

## A conversation with the assistant
- **Today**: one request, one proposal. The request carries no history, and the model is
  given the plan rather than the last thing it said. A correction to a proposal is a fresh
  request, which keeps every proposal checkable on its own.
- **Deferred**: multi-turn — "no, only the first two days" — which means keeping the turns,
  sending them back, and deciding what a proposal means when it refers to an earlier one.
- **Trigger**: wanting to narrow a proposal rather than retype the request. The cost to watch
  is that a preview is currently about exactly one sentence, which is what makes it readable.

## Further assistant operations
- **Today**: six day operations — adjust a loose product's amount, add an item, remove an
  item, swap a meal, assign a plan, clear a plan — each bounded to the dates it names; and
  the entity operations, which create a product, meal or day plan, change any field of one
  or of the targets, or sync a product from the catalogue.
- **Not offered either**: deleting anything, creating or renaming a category, recording a
  weigh-in, and syncing a product the user can name but not give a barcode for. The first
  is a decision the screens make with a confirmation of their own; the weigh-ins are kept
  out of what is sent (`docs/requirements.md` §5.3); and a sync by name would have to pick
  one of several catalogue matches for the user, which a search on the product screen does
  with the candidates in view.
- **Deferred**: the ones that need a rule rather than a field. Scaling a whole day towards a
  target has to decide what to scale and by how much. Adjusting a product that is inside a
  meal needs a decision about whether the meal is copied for that day, as the plan is. Moving
  a day's plan to another date needs to say what happens to the date it came from.
- **Trigger**: a correction that cannot be said with the six. Adding one that reduces to an
  existing day change costs a variant on `Operation`, an arm of the resolver and a tool
  schema; the preview, the confirmation and the writes are untouched.

## Showing the assistant's answer as it arrives
- **Today**: each attempt is one non-streaming request, and the chat shows a waiting state
  until the final answer is checked. Thinking at `medium` effort and the cached tools and
  system prompt keep that short, but a refused answer adds a whole attempt.
- **Deferred**: streaming the reply, and saying in the chat when an answer was refused and
  is being put right, so the wait reads as progress. It needs an event channel from
  `src-tauri` to the screen, which no command has today.
- **Trigger**: waits that still feel long at `medium` effort, or retries common enough that
  the user notices them.

## Adding to a date that has no plan
- **Today**: add-item and adjust-product-amount refuse a date with no plan, so "put this
  meal on next week" over unplanned dates needs a day plan created and assigned in the same
  answer. The prompt says so, and a refusal goes back to the model with the reason.
- **Deferred**: an add-item that gives an unplanned date a plan of its own holding just
  that item. It needs a label for a plan the user never named, and a decision about whether
  that plan is offered for other dates.
- **Trigger**: the model still getting this wrong after the retry, or the user asking for
  single items on empty days often.

## A transaction across the store traits
- **Today**: `apply_assistant_edit` checks every operation against the real data before it
  writes anything, so a proposal that no longer fits fails having touched nothing. The writes
  themselves are one store call per date, and nothing wraps them: a failure part-way through
  — a full disk, a closed database — leaves the earlier dates applied, and the log then
  records only what the call had reached.
- **Deferred**: a unit of work on `core::store` that an implementation maps to a transaction,
  so a multi-date write commits or rolls back as one.
- **Trigger**: a partial apply actually happening, or a second store whose writes are slower
  and likelier to be interrupted.
## A Play Store internal testing track
- **Today**: the signed APK is attached to a GitHub Release and Obtainium installs it, for
  both environments. Nothing is uploaded anywhere, and the acceptance build is a second
  application id rather than a track.
- **Deferred**: a Play Console account, an app signing key held by Google, and the internal
  testing track as the acceptance channel.
- **Trigger**: wanting someone other than the author to install the app, which is when
  sideloading an APK stops being a reasonable thing to ask of a tester.

## An in-app updater
- **Today**: Obtainium checks GitHub Releases and installs the APK. The app itself knows
  nothing about versions and never asks about one.
- **Deferred**: an update check inside the app. Tauri's own updater does not support
  Android, so this would be hand-written against the releases API plus a package installer
  intent.
- **Trigger**: Tauri's updater gaining Android support, or updates being missed often
  enough to be worth the first outbound connection the app makes about itself.

## Releasing to iOS
- **Today**: the release workflow builds an arm64 Android APK and nothing else. There is no
  iOS project at all — see **iOS** above for the build side of that.
- **Deferred**: a macOS runner, a signing identity and a provisioning profile, and
  TestFlight in place of Obtainium, which does not exist on iOS.
- **Trigger**: an iOS build existing locally first.

## A smoke test before a release is published
- **Today**: `check.yml` runs the unit and integration suites, and
  `crates/sqlite/tests/upgrade.rs` migrates a database an older build wrote. Nothing starts
  the built APK: a release is published on the strength of the build succeeding and the
  signature being present.
- **Deferred**: booting an emulator in the release job, installing the APK, and asserting
  the app reaches its first screen — the trigger the **End-to-end tests** entry above
  already names, now that there is a release process to hang it on.
- **Trigger**: a release that installs and then fails on launch, which no test here would
  have caught.

## Desktop bundles in CI
- **Today**: `release.yml` builds Android only. The desktop build is what `mise run dev`
  produces locally, and the frontend's contract is checked on Linux in `check.yml`.
- **Deferred**: building the Linux, macOS and Windows bundles in the release job and
  attaching them too, which needs a runner per platform.
- **Trigger**: using the app on a desktop for more than development.

## Closed by the port
Deferred decisions the predecessor carried that this version answers.

| Was deferred | Answer here |
| --- | --- |
| A migration mechanism, because `schema.sql` was applied with `CREATE TABLE IF NOT EXISTS` and a new column needed a hand-run `ALTER TABLE` | `crates/sqlite/migrations`, embedded with `sqlx::migrate!` and applied when the store opens |
| How the built frontend ships — `go:embed` in the backend, or a static server beside it | Tauri serves the built SPA into its own webview; there is no server and no deployment question |
