//! Open Food Facts as the Mealprep product catalogue.
//!
//! Open Food Facts publishes its data under the Open Database License; the application
//! credits it wherever imported data is shown.

mod entry;

use std::time::Duration;

use mealprep_core::{Entity, Error, Result, domain::CatalogueEntry, store::Catalogue};
use reqwest::{Client, StatusCode, header};
use serde::{Deserialize, de::DeserializeOwned};

use self::entry::RawProduct;

const BASE_URL: &str = "https://world.openfoodfacts.org";
const SEARCH_URL: &str = "https://search.openfoodfacts.org";
const FIELDS: &str = "code,product_name,brands_tags,image_front_small_url,nutriments,\
    nutrition_data_per,data_sources_tags,data_quality_errors_tags,data_quality_warnings_tags,\
    last_modified_t";
const PAGE_SIZE: &str = "20";
const TIMEOUT: Duration = Duration::from_secs(15);
/// Open Food Facts asks every client to identify itself.
const USER_AGENT: &str = concat!("mealprep-rs/", env!("CARGO_PKG_VERSION"));

/// Reads products from Open Food Facts.
///
/// Search goes to the dedicated search host, as the main site's own search is rate-limited;
/// single entries come from the main site.
#[derive(Clone, Debug)]
pub struct OpenFoodFacts {
    http: Client,
    base_url: String,
    search_url: String,
}

impl OpenFoodFacts {
    /// Creates a client for the public Open Food Facts hosts.
    pub fn new() -> Result<Self> {
        Self::with_hosts(BASE_URL, SEARCH_URL)
    }

    /// Creates a client for the given main and search hosts, such as a staging mirror.
    pub fn with_hosts(base_url: &str, search_url: &str) -> Result<Self> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .build()
            .map_err(|source| Error::Internal {
                reason: format!("cannot build the Open Food Facts client: {source}"),
            })?;
        let base_url = base_url.trim_end_matches('/').to_owned();
        let search_url = search_url.trim_end_matches('/').to_owned();
        Ok(Self {
            http,
            base_url,
            search_url,
        })
    }

    async fn get<T: DeserializeOwned>(&self, url: &str, query: &[(&str, &str)]) -> Result<T> {
        let response = self
            .http
            .get(url)
            .query(query)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(unavailable)?;

        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Err(Error::NotFound {
                entity: Entity::CatalogueEntry,
                id: url.to_owned(),
            });
        }
        if !status.is_success() {
            return Err(Error::CatalogueUnavailable {
                reason: format!("Open Food Facts answered {status}"),
            });
        }
        response.json().await.map_err(unavailable)
    }
}

impl Catalogue for OpenFoodFacts {
    async fn search_catalogue(&self, query: &str) -> Result<Vec<CatalogueEntry>> {
        let url = format!("{}/search", self.search_url);
        let params = [("q", query), ("page_size", PAGE_SIZE), ("fields", FIELDS)];
        let body: SearchResponse = self.get(&url, &params).await?;
        let entries = body.hits.into_iter().map(RawProduct::into_entry).collect();
        Ok(entries)
    }

    async fn get_catalogue_entry(&self, code: &str) -> Result<CatalogueEntry> {
        if code.is_empty() || !code.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(Error::Invalid {
                entity: Entity::CatalogueEntry,
                reason: format!("code {code:?} is not a barcode"),
            });
        }

        let url = format!("{}/api/v2/product/{code}", self.base_url);
        let body: ProductResponse = self.get(&url, &[("fields", FIELDS)]).await?;
        match (body.status, body.product) {
            (1, Some(product)) => Ok(product.into_entry()),
            _ => Err(Error::NotFound {
                entity: Entity::CatalogueEntry,
                id: code.to_owned(),
            }),
        }
    }
}

fn unavailable(source: reqwest::Error) -> Error {
    Error::CatalogueUnavailable {
        reason: source.to_string(),
    }
}

// serde types

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    hits: Vec<RawProduct>,
}

#[derive(Deserialize)]
struct ProductResponse {
    #[serde(default)]
    status: i64,
    product: Option<RawProduct>,
}
