use uuid::Uuid;

use super::{Mealprep, validate};
use crate::{
    Entity, Error, Result,
    domain::{Category, CategoryDraft, CategoryScope},
    store::CategoryStore,
};

impl<S, C> Mealprep<S, C>
where
    S: CategoryStore,
{
    pub async fn list_categories(&self, scope: CategoryScope) -> Result<Vec<Category>> {
        self.store.list_categories(scope).await
    }

    pub async fn get_category(&self, id: Uuid) -> Result<Category> {
        self.store.get_category(id).await
    }

    /// Creates a category whose name is unique within its scope, ignoring case.
    pub async fn create_category(&self, draft: CategoryDraft) -> Result<Category> {
        let CategoryDraft { name, scope } = draft;
        let name = validate::category_name(&name)?;
        if self.store.category_name_taken(&name, scope, None).await? {
            return Err(Error::Conflict {
                entity: Entity::Category,
                name,
            });
        }

        let draft = CategoryDraft { name, scope };
        self.store.create_category(&draft).await
    }

    /// Renames the category with the given id.
    ///
    /// Its scope stays the stored one: moving a category between the two lists would change
    /// what the labels in it are unique against.
    pub async fn rename_category(&self, id: Uuid, name: &str) -> Result<Category> {
        let stored = self.store.get_category(id).await?;
        let name = validate::category_name(name)?;
        if self
            .store
            .category_name_taken(&name, stored.scope, Some(id))
            .await?
        {
            return Err(Error::Conflict {
                entity: Entity::Category,
                name,
            });
        }
        self.store.rename_category(id, &name).await
    }

    /// Deletes the category with the given id. What it grouped keeps naming it and reads
    /// as being in a removed category.
    pub async fn delete_category(&self, id: Uuid) -> Result<()> {
        self.store.delete_category(id).await?;
        Ok(())
    }

    /// Fails unless the category with the given id exists and groups the given scope, as
    /// the given entity cannot be filed under it otherwise.
    pub(super) async fn check_category(
        &self,
        entity: Entity,
        id: Uuid,
        scope: CategoryScope,
    ) -> Result<()> {
        let category = match self.store.get_category(id).await {
            Ok(category) => category,
            Err(Error::NotFound { .. }) => {
                let reason = format!("category {id} does not exist");
                return Err(Error::Invalid { entity, reason });
            }
            Err(error) => return Err(error),
        };

        if category.scope != scope {
            let name = category.name;
            let grouped = category.scope.grouped();
            let wanted = scope.grouped();
            let reason = format!("category {name:?} groups {grouped}, not {wanted}");
            return Err(Error::Invalid { entity, reason });
        }
        Ok(())
    }
}
