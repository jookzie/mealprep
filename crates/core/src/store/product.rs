use uuid::Uuid;

use crate::{
    Result,
    domain::{Product, ProductDraft},
};

/// Persistence of the user's own products.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait ProductStore {
    /// Stores the given draft as a new product, recording the catalogue code it came from.
    async fn create_product(
        &self,
        draft: &ProductDraft,
        source_code: Option<&str>,
    ) -> Result<Product>;

    async fn get_product(&self, id: Uuid) -> Result<Product>;

    /// Lists every product, ordered by name.
    async fn list_products(&self) -> Result<Vec<Product>>;

    /// Lists those of the given products that still exist, in no particular order.
    async fn list_products_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Product>>;

    async fn update_product(&self, id: Uuid, draft: &ProductDraft) -> Result<Product>;

    async fn delete_product(&self, id: Uuid) -> Result<()>;
}
