//! The acceptance environment's demo dataset.
//!
//! Compiled in only under the `acceptance` feature, so the production build carries
//! neither the dataset nor any way to overwrite a database with it. The acceptance install
//! has its own application id and therefore its own private data directory, so nothing
//! here can reach the production database even when both are on the phone.

use std::{fs, io, path::Path};

/// The dataset `docs/demo-data.md` describes, as `mise run demo:seed` last wrote it.
///
/// Compiled into the binary rather than bundled as a Tauri resource: at 120 KB it costs
/// less than the resource-resolution path would, and a `cfg`-gated `include_bytes!` is
/// the clearest guarantee that a production binary does not contain it.
const DEMO_DATABASE: &[u8] = include_bytes!("../../demo/mealprep-demo.db");

/// Writes the demo dataset to `path` unless a database is already there.
///
/// First launch only. A later launch keeps whatever the previous acceptance session left
/// behind, because losing it on every start would make the build useless for trying
/// anything that spans two runs; `restore` is how the dataset is deliberately asked for
/// again.
pub fn seed(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::write(path, DEMO_DATABASE)
}

/// Replaces the database with the demo dataset. The application has to be restarted
/// afterwards for the new file to be read.
///
/// The store that is open while this runs holds the old file by descriptor, so it goes on
/// reading and writing the inode the old name pointed at, whatever happens to the name.
/// That is why the caller closes the application immediately: an open store cannot be
/// pointed at a different file, and until the next launch the data on screen is the data
/// that was just replaced.
pub fn restore(path: &Path) -> io::Result<()> {
    // Staged beside the database and renamed into place, so an interrupted write cannot
    // leave a half-written file where the application expects a database.
    let staged = path.with_extension("db.restoring");
    fs::write(&staged, DEMO_DATABASE)?;
    fs::rename(&staged, path)?;

    // A WAL and its index describe the database that was just replaced; left beside the
    // new file they would be replayed over it at the next open.
    for sidecar in ["db-wal", "db-shm"] {
        let sidecar = path.with_extension(sidecar);
        if sidecar.exists() {
            fs::remove_file(&sidecar)?;
        }
    }
    Ok(())
}
