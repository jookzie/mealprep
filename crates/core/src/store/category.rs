use uuid::Uuid;

use crate::{
    Result,
    domain::{Category, CategoryDraft, CategoryScope},
};

/// Persistence of the category lists.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait CategoryStore {
    async fn create_category(&self, draft: &CategoryDraft) -> Result<Category>;

    async fn get_category(&self, id: Uuid) -> Result<Category>;

    /// Lists the categories of the given scope, ordered by name.
    async fn list_categories(&self, scope: CategoryScope) -> Result<Vec<Category>>;

    async fn rename_category(&self, id: Uuid, name: &str) -> Result<Category>;

    async fn delete_category(&self, id: Uuid) -> Result<()>;

    /// Reports whether a category other than the excluded one has the given name,
    /// ignoring case, within the given scope.
    async fn category_name_taken(
        &self,
        name: &str,
        scope: CategoryScope,
        exclude: Option<Uuid>,
    ) -> Result<bool>;
}
