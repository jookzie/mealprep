# Releases

Two environments, built by `.github/workflows/release.yml` and installed on the phone with
[Obtainium](https://github.com/ImranR98/Obtainium) from GitHub Releases. There is no Play
Store listing and no in-app updater: Tauri's updater does not support Android.

| | Production | Acceptance |
| --- | --- | --- |
| Application id | `com.mertan.mealprep` | `com.mertan.mealprep.acceptance` |
| Launcher label | Mealprep | Mealprep acceptance |
| Data | the real database, kept across updates | its own private directory, seeded with `demo/mealprep-demo.db` on first launch |
| Trigger | pushing a tag `v*` | pushing to `main`, or Run workflow |
| Published as | a release, tagged `v<version>` | a prerelease, tagged `acceptance-<version>-<run number>` |
| versionCode | `1000000 + run number` | `run number` |

Both are arm64, both are signed with the same keystore, and both install at once: Android
gives each application id its own private storage, so the acceptance build can neither read
nor overwrite the real data.

The acceptance build alone is compiled with the `acceptance` cargo feature, which puts the
demo dataset in the binary, seeds it when there is no database yet, and adds the **Reset to
demo data** control at the bottom of the Targets screen — it replaces everything with the
demo dataset and closes the app, which then has to be reopened. A production build contains
none of that.

Debug builds are untouched: they keep the `.debug` id suffix and the debug signing key.

## One-time setup

1. Add the keystore to the repository's secrets, as Settings → Secrets and variables →
   Actions → New repository secret. `base64 -w0 ~/.android/mealprep-release.jks` is the
   value for the first one.

   | Secret | Value |
   | --- | --- |
   | `ANDROID_KEYSTORE_BASE64` | the release keystore, base64 with no line breaks |
   | `ANDROID_KEYSTORE_PASSWORD` | its `storePassword` |
   | `ANDROID_KEY_ALIAS` | its `keyAlias` (`mealprep`) |
   | `ANDROID_KEY_PASSWORD` | its `keyPassword` |

   The same four values are in the untracked `src-tauri/gen/android/keystore.properties`
   used locally.

2. In Obtainium, add the production app: **Add App** → this repository's URL → leave
   *Include prereleases* off → Add. It tracks tags `v*` only, and updating it keeps the
   data, because the id and the key do not change.

3. In Obtainium, add the acceptance app a second time, from the same URL: turn *Include
   prereleases* **on**, and set *Filter Release Titles by Regular Expression* to
   `acceptance`. Without the filter it would offer the production APK too, which is not
   the app it is tracking.

That is all of it; everything else is a push.

## Cutting a production release

```sh
# Bump the version in src-tauri/tauri.conf.json and package.json, then
git tag v0.2.0
git push origin v0.2.0
```

The tag has to match the version in the config: the APK's `versionName` is read from
`tauri.conf.json`, so the workflow refuses a tag that disagrees with it rather than
publishing a release whose name is not what installs.

The workflow signs the APK, attaches it to a release named `Mealprep 0.2.0`, and Obtainium
offers it on its next check. Pushing to `main` needs nothing: the acceptance prerelease is
built on every push.

## The data-safety rules

The database is a private file on the phone, so an uninstall is permanent data loss unless
an export was copied off the phone first. **Your data** on the Targets screen exports it to
`Android/data/<application id>/files/Download` and imports `mealprep-import.db` from there;
that folder is deleted with the app, so copy an export off the phone
(`adb pull /sdcard/Android/data/com.mertan.mealprep/files/Download/<file>`) for it to count as
a backup. Moving data between installs — a debug build to the release, say — is an export
or an `android:db:pull`, an `adb push` into the target's folder as `mealprep-import.db`,
and an import there.

Three things keep an update from being that uninstall:

- **The same application id.** `com.mertan.mealprep`, which is what is installed today.
  A changed id is a different app with a different, empty data directory.
- **The same signing key.** Android refuses an update signed by another key; the only way
  to install it is to uninstall first, which deletes the database. This has already cost
  the dataset once, on 2026-09-20.
- **Additive migrations only.** `crates/sqlite/migrations/` runs on every open, including
  the first open after an update. A migration adds; it does not drop a table or rewrite one
  through a copy that can lose rows, and an applied migration is never edited.
  `crates/sqlite/tests/upgrade.rs` migrates the checked-in demo database on every
  `cargo test` run, so a destructive migration fails CI rather than the phone.

The version code must also only ever rise — Android refuses an update whose code went
backwards. The two series above never overlap, and run numbers only increase, so neither
environment can go back.
