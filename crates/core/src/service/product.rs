use uuid::Uuid;

use super::{Mealprep, validate};
use crate::{
    Entity, Error, Result,
    domain::{CatalogueEntry, Product, ProductDraft},
    store::{Catalogue, ProductStore},
};

impl<S, C> Mealprep<S, C>
where
    S: ProductStore,
{
    pub async fn list_products(&self) -> Result<Vec<Product>> {
        self.store.list_products().await
    }

    pub async fn get_product(&self, id: Uuid) -> Result<Product> {
        self.store.get_product(id).await
    }

    /// Creates a product the user entered by hand.
    pub async fn create_product(&self, draft: ProductDraft) -> Result<Product> {
        let draft = validate::product(draft)?;
        self.store.create_product(&draft, None).await
    }

    /// Creates a product from the catalogue entry with the given code.
    ///
    /// The draft is the entry as the user reviewed it against the package, not the entry
    /// itself: catalogue figures are crowd-sourced and can be wrong, so what is stored is
    /// what the user confirmed. The snapshot is never refreshed from the catalogue.
    pub async fn import_product(&self, code: &str, draft: ProductDraft) -> Result<Product> {
        let code = validate::catalogue_code(code)?;
        let draft = validate::product(draft)?;
        self.store.create_product(&draft, Some(&code)).await
    }

    /// Replaces the product with the given id, keeping where it was imported from.
    pub async fn update_product(&self, id: Uuid, draft: ProductDraft) -> Result<Product> {
        let draft = validate::product(draft)?;
        self.store.update_product(id, &draft).await
    }

    /// Deletes the product with the given id. Compositions that name it keep the reference
    /// and read it as removed.
    pub async fn delete_product(&self, id: Uuid) -> Result<()> {
        self.store.delete_product(id).await?;
        Ok(())
    }

    /// Fails when one of the given products does not exist, as the given entity cannot be
    /// saved naming it.
    pub(super) async fn check_products_exist(&self, entity: Entity, ids: &[Uuid]) -> Result<()> {
        let found = self.store.list_products_by_ids(ids).await?;
        let missing = ids
            .iter()
            .find(|id| found.iter().all(|product| product.id != **id));
        if let Some(missing) = missing {
            let reason = format!("product {missing} does not exist");
            return Err(Error::Invalid { entity, reason });
        }
        Ok(())
    }
}

impl<S, C> Mealprep<S, C>
where
    C: Catalogue,
{
    /// Searches the external catalogue by the given free text.
    pub async fn search_catalogue(&self, query: &str) -> Result<Vec<CatalogueEntry>> {
        let query = query.trim();
        if query.is_empty() {
            return Err(Error::Invalid {
                entity: Entity::CatalogueEntry,
                reason: "the search query is empty".to_owned(),
            });
        }
        self.catalogue.search_catalogue(query).await
    }

    /// Fetches the catalogue entry with the given code, such as a scanned barcode, for
    /// the user to review before importing it.
    pub async fn get_catalogue_entry(&self, code: &str) -> Result<CatalogueEntry> {
        let code = validate::catalogue_code(code)?;
        self.catalogue.get_catalogue_entry(&code).await
    }
}
