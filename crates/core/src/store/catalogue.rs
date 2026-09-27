use crate::{Result, domain::CatalogueEntry};

/// The external catalogue products are imported from.
#[trait_variant::make(Send)]
pub trait Catalogue {
    /// Searches for entries whose names match the given query.
    async fn search_catalogue(&self, query: &str) -> Result<Vec<CatalogueEntry>>;

    /// Fetches the entry with the given code.
    async fn get_catalogue_entry(&self, code: &str) -> Result<CatalogueEntry>;
}
