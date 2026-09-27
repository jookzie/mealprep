//! Every platform but Android: there is no Health Connect to read.

use std::marker::PhantomData;

use mealprep_core::{
    Error, Result,
    domain::{HealthAvailability, HealthImport},
};
use tauri::{Runtime, plugin::PluginApi};
use time::Date;

pub(crate) struct Source<R: Runtime>(PhantomData<fn() -> R>);

impl<R: Runtime> Source<R> {
    pub(crate) fn register(
        _api: PluginApi<R, ()>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self(PhantomData))
    }

    pub(crate) async fn availability(&self) -> Result<HealthAvailability> {
        Ok(HealthAvailability::Unsupported)
    }

    pub(crate) async fn connected(&self) -> Result<bool> {
        Ok(false)
    }

    pub(crate) async fn connect(&self) -> Result<bool> {
        Err(unsupported())
    }

    pub(crate) async fn install(&self) -> Result<()> {
        Err(unsupported())
    }

    pub(crate) async fn read(&self, _from: Date, _to: Date) -> Result<HealthImport> {
        Err(unsupported())
    }
}

fn unsupported() -> Error {
    Error::HealthUnavailable {
        reason: "Health Connect exists on Android only".to_owned(),
    }
}
