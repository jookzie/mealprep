//! Health Connect as a source of health data.
//!
//! A Tauri plugin over the Android client in `android/`. The Kotlin side reads records and
//! reports what it found; this crate turns that into [`HealthImport`], and the rules in
//! `mealprep-core` do everything else. Every other platform has no Health Connect, and says
//! so rather than failing at build time.

#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod unsupported;
mod wire;

use mealprep_core::{
    Result,
    domain::{HealthAvailability, HealthImport},
};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};
use time::Date;

#[cfg(target_os = "android")]
use self::android::Source;
#[cfg(not(target_os = "android"))]
use self::unsupported::Source;

/// Access to Health Connect, managed by the application once the plugin is initialised.
pub struct HealthConnect<R: Runtime>(Source<R>);

impl<R: Runtime> HealthConnect<R> {
    /// Whether the device has a usable Health Connect.
    pub async fn availability(&self) -> Result<HealthAvailability> {
        self.0.availability().await
    }

    /// Whether the user has granted at least one of the permissions the import reads with.
    ///
    /// At least one rather than all: someone may reasonably decline sleep and keep heart
    /// rate, and the import reads whatever it was allowed to.
    pub async fn connected(&self) -> Result<bool> {
        self.0.connected().await
    }

    /// Shows Health Connect's permission screen and answers whether access was granted.
    pub async fn connect(&self) -> Result<bool> {
        self.0.connect().await
    }

    /// Opens the Play Store on Health Connect, for a device that must install or update it.
    pub async fn install(&self) -> Result<()> {
        self.0.install().await
    }

    /// Reads everything granted for the local days `from` to `to` inclusive.
    pub async fn read(&self, from: Date, to: Date) -> Result<HealthImport> {
        self.0.read(from, to).await
    }
}

/// Reaches [`HealthConnect`] from anything that can reach the application's state.
pub trait HealthConnectExt<R: Runtime> {
    fn health_connect(&self) -> &HealthConnect<R>;
}

impl<R: Runtime, T: Manager<R>> HealthConnectExt<R> for T {
    fn health_connect(&self) -> &HealthConnect<R> {
        self.state::<HealthConnect<R>>().inner()
    }
}

/// Initialises the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("health-connect")
        .setup(|app, api| {
            app.manage(HealthConnect(Source::register(api)?));
            Ok(())
        })
        .build()
}
