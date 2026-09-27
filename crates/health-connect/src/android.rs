//! Android: the Kotlin plugin in `android/` does the reading.

use mealprep_core::{
    Error, Result,
    domain::{self, HealthAvailability, HealthImport},
};
use tauri::{
    Runtime,
    plugin::{PluginApi, PluginHandle, mobile::PluginInvokeError},
};
use time::Date;

use crate::wire::{Connected, ReadArgs, ReadResponse, Status};

const PLUGIN_IDENTIFIER: &str = "com.mertan.mealprep.health";
const PLUGIN_CLASS: &str = "HealthConnectPlugin";

pub(crate) struct Source<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Source<R> {
    pub(crate) fn register(
        api: PluginApi<R, ()>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, PLUGIN_CLASS)?;
        Ok(Self(handle))
    }

    pub(crate) async fn availability(&self) -> Result<HealthAvailability> {
        let status: Status = self.run("status", ()).await?;
        Ok(status.availability())
    }

    pub(crate) async fn connected(&self) -> Result<bool> {
        let status: Status = self.run("status", ()).await?;
        Ok(status.connected)
    }

    pub(crate) async fn connect(&self) -> Result<bool> {
        let connected: Connected = self.run("connect", ()).await?;
        Ok(connected.connected)
    }

    pub(crate) async fn install(&self) -> Result<()> {
        self.run::<()>("install", ()).await
    }

    pub(crate) async fn read(&self, from: Date, to: Date) -> Result<HealthImport> {
        let args = ReadArgs {
            from: domain::format_iso_date(from),
            to: domain::format_iso_date(to),
        };
        let response: ReadResponse = self.run("read", args).await?;
        response.into_import()
    }

    async fn run<T: serde::de::DeserializeOwned>(
        &self,
        command: &str,
        payload: impl serde::Serialize,
    ) -> Result<T> {
        self.0
            .run_mobile_plugin_async(command, payload)
            .await
            .map_err(unavailable)
    }
}

fn unavailable(error: PluginInvokeError) -> Error {
    Error::HealthUnavailable {
        reason: error.to_string(),
    }
}
